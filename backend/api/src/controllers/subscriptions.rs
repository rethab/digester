use lib_db as db;
use lib_messaging as messaging;

use super::super::subscriptions;
use super::common::*;
use chrono::naive::NaiveTime;
use chrono::Utc;
use chrono_tz::Tz;
use db::{ChannelType, Day, Frequency, Timezone};
use either::{Left, Right};
use messaging::sendgrid::pending_subscriptions;
use rocket::Rocket;
use rocket_contrib::json::{Json, JsonValue};
use std::str::FromStr;
use uuid::Uuid;

pub fn mount(rocket: Rocket) -> Rocket {
    rocket.mount(
        "/subscriptions",
        routes![
            list,
            add,
            show,
            update,
            delete,
            add_pending,
            activate_pending
        ],
    )
}

#[derive(Deserialize, Debug, PartialEq)]
struct NewSubscription {
    #[serde(rename = "channelId")]
    channel_id: i32,
    #[serde(rename = "channelType")]
    channel_type: SearchChannelType,
    frequency: Frequency,
    day: Option<Day>,
    time: NaiveTime,
}

#[derive(Deserialize, Debug, PartialEq)]
struct NewPendingSubscription {
    email: String,
    timezone: String,
    #[serde(rename = "channelId")]
    channel_id: i32,
    #[serde(rename = "channelType")]
    channel_type: SearchChannelType,
    frequency: Frequency,
    day: Option<Day>,
    time: NaiveTime,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
struct Subscription {
    id: i32,
    #[serde(rename = "name")]
    name: String,
    #[serde(rename = "channelType")]
    channel_type: SearchChannelType,
    #[serde(rename = "channelId")]
    channel_id: i32,
    summary: Option<String>,
    #[serde(rename = "channelLink")]
    channel_link: Option<String>,
    frequency: Frequency,
    day: Option<Day>,
    time: NaiveTime,
}

impl Into<JsonResponse> for Subscription {
    fn into(self) -> JsonResponse {
        match serde_json::to_value(self.clone()) {
            Ok(v) => JsonResponse::Ok(JsonValue(v)),
            Err(err) => {
                eprintln!(
                    "Failed to convert Subscription {:?} into JsonResponse: {:?}",
                    self, err
                );
                JsonResponse::InternalServerError
            }
        }
    }
}

impl Into<JsonResponse> for Vec<Subscription> {
    fn into(self) -> JsonResponse {
        match serde_json::to_value(self) {
            Ok(v) => JsonResponse::Ok(JsonValue(v)),
            Err(err) => {
                eprintln!(
                    "Failed to convert Vec<Subscription> into JsonResponse: {:?}",
                    err
                );
                JsonResponse::InternalServerError
            }
        }
    }
}

impl Subscription {
    fn from_db_channel(sub: db::Subscription, chan: db::Channel) -> Subscription {
        Subscription {
            id: sub.id,
            name: chan.name.clone(),
            channel_type: SearchChannelType::from_db(chan.channel_type),
            channel_id: chan.id,
            summary: None,
            channel_link: Some(chan.link),
            frequency: sub.frequency,
            day: sub.day,
            time: sub.time,
        }
    }
    fn from_db_list(
        sub: db::Subscription,
        list: db::List,
        channels: Vec<db::Channel>,
    ) -> Subscription {
        Subscription {
            id: sub.id,
            name: list.name.clone(),
            channel_type: SearchChannelType::List,
            channel_id: list.id,
            summary: Some(format!("{} channels", channels.len())),
            channel_link: None,
            frequency: sub.frequency,
            day: sub.day,
            time: sub.time,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
enum SearchChannelType {
    GithubRelease,
    RssFeed,
    Twitter,
    List,
}

impl SearchChannelType {
    fn from_db(channel_type: db::ChannelType) -> SearchChannelType {
        match channel_type {
            ChannelType::GithubRelease => SearchChannelType::GithubRelease,
            ChannelType::RssFeed => SearchChannelType::RssFeed,
            ChannelType::Twitter => SearchChannelType::Twitter,
        }
    }
}

#[get("/")]
fn list(session: Protected, db: DigesterDbConn) -> JsonResponse {
    let user_id = session.0.user_id;
    match db::subscriptions_find_by_user_id(&db, user_id.into()) {
        Err(err) => {
            eprintln!(
                "Failed to load subscriptions for user {}: {:?}",
                user_id, err
            );
            JsonResponse::InternalServerError
        }
        Ok(subs) => {
            let mut collected = Vec::new();
            for (sub, chan_or_list) in subs {
                match chan_or_list {
                    Left(channel) => collected.push(Subscription::from_db_channel(sub, channel)),
                    Right(list) => match db::channels_find_by_list_id(&db, list.id) {
                        Ok(channels) => {
                            collected.push(Subscription::from_db_list(sub, list, channels))
                        }
                        Err(err) => {
                            eprintln!("Failed to fetch channels for list {}: {}", list.id, err);
                        }
                    },
                }
            }
            collected.into()
        }
    }
}

#[post("/add", data = "<new_subscription>")]
fn add(
    session: Protected,
    db: DigesterDbConn,
    new_subscription: Json<NewSubscription>,
) -> JsonResponse {
    use subscriptions::AddError::*;

    let channel_type = match new_subscription.channel_type {
        SearchChannelType::List => {
            subscriptions::SearchChannelType::List(new_subscription.channel_id)
        }
        SearchChannelType::GithubRelease
        | SearchChannelType::RssFeed
        | SearchChannelType::Twitter => {
            subscriptions::SearchChannelType::Channel(new_subscription.channel_id)
        }
    };
    match subscriptions::add(
        session.0.user_id,
        &db,
        channel_type,
        new_subscription.frequency.clone(),
        new_subscription.day.clone(),
        new_subscription.time,
    ) {
        Err(Unknown(msg)) => {
            eprintln!("Failed to add subscription: {}", msg);
            JsonResponse::InternalServerError
        }
        Err(NotFound(msg)) => JsonResponse::BadRequest(msg),
        Err(AlreadyExists) => JsonResponse::BadRequest("Subscription already exists".into()),
        Ok((sub, c_or_l)) => match c_or_l {
            Left(channel) => Subscription::from_db_channel(sub, channel).into(),
            Right((list, channels)) => Subscription::from_db_list(sub, list, channels).into(),
        },
    }
}

#[post("/add_pending", data = "<new_sub>")]
fn add_pending(db: DigesterDbConn, new_sub: Json<NewPendingSubscription>) -> JsonResponse {
    match new_sub.channel_type {
        SearchChannelType::List => {
            let list_id = new_sub.channel_id;
            match db::lists_find_by_id(&db, list_id) {
                Err(err) => {
                    eprintln!("Failed to fetch list by id '{}': {:?}", list_id, err);
                    return JsonResponse::BadRequest("list does not exist".into());
                }
                Ok(Some((list, _))) => list,
                Ok(None) => return JsonResponse::BadRequest("list does not exist".into()),
            };

            match validate_pending_subscription(&new_sub) {
                Ok(new_pending_sub) => {
                    let token = new_pending_sub.token.clone();
                    match db::pending_subscriptions_insert(&db, new_pending_sub) {
                        Ok(pending_sub) => {
                            send_activation_email(&db, &pending_sub, &token);
                            JsonResponse::Ok(json!({}))
                        }
                        Err(db::InsertError::Unknown(err)) => {
                            eprintln!(
                                "Failed to insert pending subscription {:?}: {:?}",
                                new_sub, err
                            );
                            JsonResponse::InternalServerError
                        }
                        Err(db::InsertError::Duplicate) => {
                            JsonResponse::BadRequest("Subscription already exists".into())
                        }
                    }
                }
                Err(err) => JsonResponse::BadRequest(err),
            }
        }
        _ => {
            eprintln!("Called add anonymous for unsupported channel type");
            JsonResponse::BadRequest("Only lists are supported".into())
        }
    }
}

fn validate_pending_subscription(
    new_sub: &NewPendingSubscription,
) -> Result<db::NewPendingSubscription, String> {
    match (
        validate_email(&new_sub.email),
        validate_timezone(&new_sub.timezone),
    ) {
        (Ok(email), Ok(timezone)) => {
            let token: String = Uuid::new_v4()
                .to_simple()
                .encode_lower(&mut Uuid::encode_buffer())
                .to_owned();

            Ok(db::NewPendingSubscription {
                email,
                timezone,
                list_id: new_sub.channel_id,
                token,
                frequency: new_sub.frequency.clone(),
                day: new_sub.day.clone(),
                time: new_sub.time,
            })
        }
        (email_err, tz_err) => {
            let mut err_str = String::new();
            if let Err(email_err) = email_err {
                err_str.push_str(&email_err);
            }
            if let Err(tz_err) = tz_err {
                if !err_str.is_empty() {
                    err_str.push_str(", ")
                }
                err_str.push_str(&tz_err)
            }
            Err(err_str)
        }
    }
}

fn validate_email(email: &str) -> Result<String, String> {
    let err = Err("Not a valid e-mail".into());
    let mut cleaned = email.to_ascii_lowercase();
    cleaned = cleaned.trim().to_string();
    let parts: Vec<&str> = cleaned.split('@').collect();
    if parts.len() != 2 {
        return err;
    }

    let name = parts[0];
    let domain = parts[1];

    if name.is_empty() {
        return err;
    }

    let parts: Vec<&str> = domain.split('.').collect();
    if parts.len() < 2 {
        return err;
    }

    if parts.iter().any(|p| p.is_empty()) {
        return err;
    }

    Ok(cleaned)
}

fn validate_timezone(timezone: &str) -> Result<Timezone, String> {
    Tz::from_str(timezone)
        .map(Timezone)
        .map_err(|_| format!("Not a valid timezone: {}", timezone))
}

fn send_activation_email(db: &DigesterDbConn, pending_sub: &db::PendingSubscription, token: &str) {
    match pending_subscriptions::send_activation_email(&pending_sub.email, token) {
        Ok(()) => match db::pending_subscriptions_set_sent(db, pending_sub, Utc::now()) {
            Ok(()) => {
                println!(
                    "Successfully sent pending subscription e-mail for {}",
                    pending_sub.id
                );
            }
            Err(err) => {
                eprintln!(
                    "Failed to update token of pending subscription {} in database: {:?}",
                    pending_sub.id, err
                );
            }
        },
        Err(err) => {
            eprintln!(
                "Failed to send activation e-mail to {}: {:?}",
                pending_sub.email, err
            );
        }
    }
}

#[post("/activate/<token>")]
fn activate_pending(db: DigesterDbConn, token: String) -> JsonResponse {
    let pending_sub = match db::pending_subscriptions_find_by_token(&db, &token) {
        Ok(Some(ps)) => ps,
        Ok(None) => return JsonResponse::NotFound,
        Err(err) => {
            eprintln!(
                "Failed to fetch pending subscription with token {} from db: {:?}",
                token, err
            );
            return JsonResponse::InternalServerError;
        }
    };

    let new_sub = db::NewSubscription {
        email: pending_sub.email.clone(),
        timezone: Some(pending_sub.timezone.clone()),
        channel_id: None,
        list_id: Some(pending_sub.list_id),
        user_id: None,
        frequency: pending_sub.frequency.clone(),
        day: pending_sub.day.clone(),
        time: pending_sub.time,
    };

    match db::subscriptions_insert(&db, new_sub) {
        Err(db::InsertError::Duplicate) => {
            JsonResponse::BadRequest("Subscription already exists".into())
        }
        Err(db::InsertError::Unknown(err)) => {
            eprintln!(
                "Failed to insert new subscription for pending subscription {}: {:?}",
                pending_sub.id, err
            );
            JsonResponse::InternalServerError
        }
        Ok(sub) => {
            let id = pending_sub.id;
            match db::pending_subscriptions_delete(&db, pending_sub) {
                Ok(()) => {
                    println!("Successfully activated subscription {}", sub.id);
                    JsonResponse::Ok(json!({}))
                }
                Err(err) => {
                    eprintln!(
                        "Failed to delete pending subscription with id {}: {:?}",
                        id, err
                    );
                    JsonResponse::InternalServerError
                }
            }
        }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
struct UpdatedSubscription {
    frequency: Frequency,
    day: Option<Day>,
    time: NaiveTime,
}

#[get("/<id>")]
fn show(session: Protected, db: DigesterDbConn, id: i32) -> JsonResponse {
    let (sub, channel_or_list) =
        match db::subscriptions_find_by_id_user_id(&db.0, id, session.0.user_id.into()) {
            Ok(Some((sub, channel_or_list))) => (sub, channel_or_list),
            Ok(None) => return JsonResponse::NotFound,
            Err(_) => return JsonResponse::InternalServerError,
        };

    match channel_or_list {
        Left(channel) => Subscription::from_db_channel(sub, channel).into(),
        Right(list) => {
            let channels = match db::channels_find_by_list_id(&db, list.id) {
                Ok(channels) => channels,
                Err(err) => {
                    eprintln!("Failed to find channels by list id {}: {:?}", list.id, err);
                    return JsonResponse::InternalServerError;
                }
            };
            Subscription::from_db_list(sub, list, channels).into()
        }
    }
}

#[put("/<id>", data = "<updated_subscription>")]
fn update(
    session: Protected,
    db: DigesterDbConn,
    id: i32,
    updated_subscription: Json<UpdatedSubscription>,
) -> JsonResponse {
    let (original, channel_or_list) =
        match db::subscriptions_find_by_id_user_id(&db.0, id, session.0.user_id.into()) {
            Ok(Some((sub, channel_or_list))) => (sub, channel_or_list),
            Ok(None) => return JsonResponse::NotFound,
            Err(_) => return JsonResponse::InternalServerError,
        };

    match update_subscription(&db, updated_subscription.0, original) {
        Ok(sub) => match channel_or_list {
            Left(channel) => Subscription::from_db_channel(sub, channel).into(),
            Right(list) => {
                let channels = match db::channels_find_by_list_id(&db, list.id) {
                    Ok(channels) => channels,
                    Err(err) => {
                        eprintln!("Failed to find channels by list id {}: {:?}", list.id, err);
                        return JsonResponse::InternalServerError;
                    }
                };
                Subscription::from_db_list(sub, list, channels).into()
            }
        },
        Err(_) => JsonResponse::InternalServerError,
    }
}

#[delete("/<id>")]
fn delete(session: Protected, db: DigesterDbConn, id: i32) -> JsonResponse {
    let sub = match db::subscriptions_find_by_id_user_id(&db.0, id, session.0.user_id.into()) {
        Ok(Some((sub, _))) => sub,
        Ok(None) => return JsonResponse::NotFound,
        Err(_) => return JsonResponse::InternalServerError,
    };
    match subscriptions::delete(&db, sub.id) {
        Ok(()) => JsonResponse::Ok(json!({})),
        Err(err) => {
            eprintln!(
                "Failed to delete subscription {} for user {}: {}",
                id, session.0.user_id, err
            );
            JsonResponse::InternalServerError
        }
    }
}

fn update_subscription(
    conn: &DigesterDbConn,
    updated: UpdatedSubscription,
    original: db::Subscription,
) -> Result<db::Subscription, String> {
    let db_sub = db::Subscription {
        frequency: updated.frequency,
        day: updated.day,
        time: updated.time,
        ..original
    };
    db::subscriptions_update(&conn.0, db_sub).map(|sub| {
        if let Err(err) = db::digests_remove_unsent_for_subscription(&conn.0, &sub) {
            println!(
                "Failed to remove digest after updating subscription: {:?}",
                err
            );
        };
        sub
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_subscription() {
        let sub: NewSubscription = serde_json::from_str(
            r#"{
            "channelId":1,
            "channelType": "RssFeed",
            "frequency":"Weekly",
            "day":"Sat",
            "time":"09:00:00.00"
        }"#,
        )
        .expect("Failed to parse");
        let exp = NewSubscription {
            channel_id: 1,
            channel_type: SearchChannelType::RssFeed,
            frequency: Frequency::Weekly,
            day: Some(Day::Sat),
            time: NaiveTime::from_hms_milli(9, 0, 0, 0),
        };

        assert_eq!(exp, sub);
    }

    #[test]
    fn valid_emails() {
        assert!(validate_email("test@test.ch").is_ok());
        assert!(validate_email("test@test.test.ch").is_ok());
        assert!(validate_email("TEST@TEST.TEST.CH").is_ok());
        assert!(validate_email("a@b.c").is_ok());
        assert!(validate_email("rh+xy@kl.mu").is_ok());
        assert!(validate_email("mylongemail@mylongdomain.mytld").is_ok());
    }

    #[test]
    fn invalid_emails() {
        assert!(validate_email("localhost").is_err());
        assert!(validate_email("me@me").is_err());
        assert!(validate_email("me@me@me").is_err());
        assert!(validate_email("me@me.me@me").is_err());
        assert!(validate_email("").is_err());
        assert!(validate_email(" @ . ").is_err());
        assert!(validate_email("test@test").is_err());
        assert!(validate_email("gmail.com").is_err());
    }

    #[test]
    fn valid_timezones() {
        assert!(validate_timezone("Europe/Amsterdam").is_ok());
    }

    #[test]
    fn invalid_timezones() {
        assert!(validate_timezone("Europe/Rotterdam").is_err());
        assert!(validate_timezone("Europe/").is_err());
        assert!(validate_timezone("Europe").is_err());
        assert!(validate_timezone("/").is_err());
        assert!(validate_timezone(" ").is_err());
        assert!(validate_timezone("").is_err());
    }
}
