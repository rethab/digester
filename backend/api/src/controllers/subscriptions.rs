use lib_channels::github_release::GithubRelease;
use lib_channels::{Channel, ValidationError};
use lib_db as db;

use super::common::*;
use chrono::naive::NaiveTime;
use db::{ChannelType, Day, Frequency};
use rocket::http::RawStr;
use rocket::request::FromFormValue;
use rocket::{Rocket, State};
use rocket_contrib::json::{Json, JsonValue};

pub fn mount(rocket: Rocket) -> Rocket {
    rocket.mount("/subscriptions", routes![list, search, add, update])
}

pub struct GithubApiToken(pub String);

#[derive(Deserialize, Debug, PartialEq)]
struct NewSubscription {
    #[serde(rename = "channelName")]
    channel_name: String,
    #[serde(rename = "channelType")]
    channel_type: ChannelType,
    frequency: Frequency,
    day: Option<Day>,
    time: NaiveTime,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
struct Subscription {
    id: i32,
    #[serde(rename = "channelName")]
    channel_name: String,
    #[serde(rename = "channelType")]
    channel_type: ChannelType,
    frequency: Frequency,
    day: Option<Day>,
    time: NaiveTime,
}

impl NewSubscription {
    fn with_name(self, name: String) -> Self {
        Self {
            channel_name: name,
            channel_type: self.channel_type,
            frequency: self.frequency,
            day: self.day,
            time: self.time,
        }
    }
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
    fn from_db(sub: db::Subscription, chan: db::Channel) -> Subscription {
        Subscription {
            id: sub.id,
            channel_name: chan.name,
            channel_type: chan.channel_type,
            frequency: sub.frequency,
            day: sub.day,
            time: sub.time,
        }
    }
}

struct SearchChannelType(ChannelType);

impl<'v> FromFormValue<'v> for SearchChannelType {
    type Error = String;
    fn from_form_value(form_value: &'v RawStr) -> Result<SearchChannelType, String> {
        serde_json::de::from_str::<ChannelType>(&form_value)
            .map(|ct| SearchChannelType(ct))
            .map_err(|err| {
                format!(
                    "Failed to deserialize {} into SearchChannelType: {:?}",
                    &form_value, err
                )
            })
    }
}

#[get("/")]
fn list(session: Protected, db: DigesterDbConn) -> JsonResponse {
    match db::subscriptions_find_by_user_id(&db, session.0.user_id) {
        Err(_) => JsonResponse::InternalServerError,
        Ok(subs) => subs
            .into_iter()
            .map(|(sub, chan)| Subscription::from_db(sub, chan))
            .collect::<Vec<Subscription>>()
            .into(),
    }
}

#[get("/search?<channel_type>&<query>")]
fn search(
    _session: Protected,
    db: DigesterDbConn,
    channel_type: SearchChannelType,
    query: &RawStr,
) -> JsonResponse {
    unimplemented!()
}

#[post("/add", data = "<new_subscription>")]
fn add(
    session: Protected,
    db: DigesterDbConn,
    github_api_token: State<GithubApiToken>,
    new_subscription: Json<NewSubscription>,
) -> JsonResponse {
    let identity = match db::identities_find_by_user_id(&db, session.0.user_id) {
        Ok(identity) => identity,
        Err(_) => return JsonResponse::InternalServerError,
    };

    let valid_subscription = match validate(new_subscription.0, &github_api_token) {
        Ok(valid) => valid,
        Err(msg) => return JsonResponse::BadRequest(msg),
    };

    let channel = match insert_channel_if_not_exists(&db, &valid_subscription) {
        Ok(c) => c,
        Err(err) => {
            eprintln!(
                "Failed to insert new channel for subscription {:?}: {:?}",
                valid_subscription, err
            );
            return JsonResponse::InternalServerError;
        }
    };

    match insert_subscription(&db, valid_subscription, &channel, &identity) {
        Ok(sub) => Subscription::from_db(sub, channel).into(),
        Err(db::InsertError::Duplicate) => {
            JsonResponse::BadRequest("Already subscribed to repository".to_owned())
        }
        Err(db::InsertError::Unknown) => JsonResponse::InternalServerError,
    }
}

#[derive(Deserialize, Debug, PartialEq)]
struct UpdatedSubscription {
    frequency: Frequency,
    day: Option<Day>,
    time: NaiveTime,
}

#[put("/<id>", data = "<updated_subscription>")]
fn update(
    session: Protected,
    db: DigesterDbConn,
    id: i32,
    updated_subscription: Json<UpdatedSubscription>,
) -> JsonResponse {
    let (original, channel) = match db::subscriptions_find_by_id(&db.0, id, session.0.user_id) {
        Ok(Some((sub, chan))) => (sub, chan),
        Ok(None) => return JsonResponse::NotFound,
        Err(_) => return JsonResponse::InternalServerError,
    };

    match update_subscription(&db, updated_subscription.0, original) {
        Ok(sub) => Subscription::from_db(sub, channel).into(),
        Err(_) => JsonResponse::InternalServerError,
    }
}

fn validate(sub: NewSubscription, gh_token: &GithubApiToken) -> Result<NewSubscription, String> {
    match sub.channel_type {
        ChannelType::GithubRelease => {
            // todo: if we already know this repo, no need to call github
            let github: GithubRelease =
                GithubRelease::new(&gh_token.0).map_err(|_| "Unknown problem".to_owned())?;
            github
                .sanitize(&sub.channel_name)
                .map_err(|err| {
                    eprintln!("Invalid channel: {}", err);
                    "Invalid channel name".into()
                })
                .and_then(|repo| {
                    github
                        .validate(repo)
                        .map(|repo_name| sub.with_name(repo_name.0))
                        .map_err(|err| match err {
                            ValidationError::ChannelNotFound => "Repository does not exist".into(),
                            ValidationError::TechnicalError => "Unknown error".into(),
                        })
                })
        }
        ChannelType::RssFeed => unimplemented!(),
    }
}

fn insert_channel_if_not_exists(
    conn: &DigesterDbConn,
    sub: &NewSubscription,
) -> Result<db::Channel, String> {
    let new_channel = db::NewChannel {
        name: sub.channel_name.clone(),
        channel_type: sub.channel_type,
    };
    db::channels_insert_if_not_exists(&conn.0, new_channel)
}

fn insert_subscription(
    conn: &DigesterDbConn,
    sub: NewSubscription,
    chan: &db::Channel,
    identity: &db::Identity,
) -> Result<db::Subscription, db::InsertError> {
    let new_subscription = db::NewSubscription {
        email: identity.email.clone(),
        channel_id: chan.id,
        user_id: identity.user_id,
        frequency: sub.frequency,
        day: sub.day,
        time: sub.time,
    };
    db::subscriptions_insert(&conn.0, new_subscription)
}

fn update_subscription(
    conn: &DigesterDbConn,
    updated: UpdatedSubscription,
    original: db::Subscription,
) -> Result<db::Subscription, String> {
    let db_sub = db::Subscription {
        id: original.id,
        email: original.email,
        channel_id: original.channel_id,
        user_id: original.user_id,
        frequency: updated.frequency,
        day: updated.day,
        time: updated.time,
        inserted: original.inserted,
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
            "channelName":"rethab/dotfiles",
            "channelType":"GithubRelease",
            "frequency":"Weekly",
            "day":"Sat",
            "time":"09:00:00.00"
        }"#,
        )
        .expect("Failed to parse");
        let exp = NewSubscription {
            channel_name: "rethab/dotfiles".into(),
            channel_type: ChannelType::GithubRelease,
            frequency: Frequency::Weekly,
            day: Some(Day::Sat),
            time: NaiveTime::from_hms_milli(9, 0, 0, 0),
        };

        assert_eq!(exp, sub);
    }
}
