use lib_channels as channels;
use lib_db as db;

use super::super::subscriptions::search;
use super::common::*;
use channels::github_release::GithubRelease;
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
    #[serde(rename = "channelId")]
    channel_id: i32,
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
    #[serde(rename = "channelId")]
    channel_id: i32,
    #[serde(rename = "channelLink")]
    channel_link: String,
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
    fn from_db(sub: db::Subscription, chan: db::Channel) -> Subscription {
        Subscription {
            id: sub.id,
            channel_name: chan.name.clone(),
            channel_type: chan.channel_type,
            channel_id: chan.id,
            channel_link: chan.link,
            frequency: sub.frequency,
            day: sub.day,
            time: sub.time,
        }
    }
}

#[derive(Serialize)]
struct Channel {
    id: i32,
    channel_type: ChannelType,
    name: String,
    link: String,
}

impl Channel {
    fn from_db(c: db::Channel) -> Self {
        Self {
            id: c.id,
            channel_type: c.channel_type,
            name: c.name,
            link: c.link,
        }
    }
}

struct SearchChannelType(ChannelType);

impl<'v> FromFormValue<'v> for SearchChannelType {
    type Error = String;
    fn from_form_value(form_value: &'v RawStr) -> Result<SearchChannelType, String> {
        match form_value.as_str() {
            "GithubRelease" => Ok(SearchChannelType(ChannelType::GithubRelease)),
            "RssFeed" => Ok(SearchChannelType(ChannelType::RssFeed)),
            other => Err(format!("Invalid channel type: {}", other)),
        }
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
    _r: RateLimited,
    db: DigesterDbConn,
    channel_type: SearchChannelType,
    gh_token: State<GithubApiToken>,
    query: &RawStr,
) -> JsonResponse {
    let query = match query.url_decode() {
        Ok(query) => query,
        Err(err) => {
            eprintln!("Failed to decode raw string: '{}': {:?}", query, err);
            return JsonResponse::InternalServerError;
        }
    };

    // todo only initialize this if needed
    let github: GithubRelease = match GithubRelease::new(&gh_token.0) {
        Ok(github) => github,
        Err(err) => {
            eprintln!("Failed to resolve github client: {:?}", err);
            return JsonResponse::InternalServerError;
        }
    };

    let db_channel_type = channel_type.0;

    let channel_type = match db_channel_type {
        db::ChannelType::GithubRelease => channels::ChannelType::GithubRelease,
        db::ChannelType::RssFeed => channels::ChannelType::RssFeed,
    };
    let channel = channels::factory(channel_type, &github);

    use search::SearchError::*;
    match search::search(&db.0, db_channel_type, channel, &query) {
        Err(Unknown) => JsonResponse::InternalServerError,
        Err(InvalidInput) => JsonResponse::BadRequest("Invalid Input".into()),
        Err(NotFound) => JsonResponse::BadRequest(
            "This does not exist. Are you sure the input is correct?".into(),
        ),
        Err(Timeout) => JsonResponse::BadRequest(
            "We could not fetch your feed fast enough. Please try again later.".into(),
        ),
        Ok(channels) => {
            let channels = channels
                .into_iter()
                .map(Channel::from_db)
                .collect::<Vec<Channel>>();
            JsonResponse::Ok(json!({ "channels": channels }))
        }
    }
}

#[post("/add", data = "<new_subscription>")]
fn add(
    session: Protected,
    db: DigesterDbConn,
    new_subscription: Json<NewSubscription>,
) -> JsonResponse {
    let identity = match db::identities_find_by_user_id(&db, session.0.user_id) {
        Ok(identity) => identity,
        Err(_) => return JsonResponse::InternalServerError,
    };

    let chan_id = new_subscription.channel_id;

    let channel = match db::channels_find_by_id(&db, chan_id) {
        Err(err) => {
            eprintln!("Failed to fetch channel by id '{}': {:?}", chan_id, err);
            return JsonResponse::BadRequest("channel does not exist".into());
        }
        Ok(channel) => channel,
    };

    match insert_subscription(&db, &new_subscription, &channel, &identity) {
        Ok(sub) => Subscription::from_db(sub, channel).into(),
        Err(db::InsertError::Duplicate) => {
            JsonResponse::BadRequest("Subscription already exists".to_owned())
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

fn insert_subscription(
    conn: &DigesterDbConn,
    sub: &NewSubscription,
    chan: &db::Channel,
    identity: &db::Identity,
) -> Result<db::Subscription, db::InsertError> {
    let new_subscription = db::NewSubscription {
        email: identity.email.clone(),
        channel_id: chan.id,
        user_id: identity.user_id,
        frequency: sub.frequency.clone(),
        day: sub.day.clone(),
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
            "channelId":1,
            "frequency":"Weekly",
            "day":"Sat",
            "time":"09:00:00.00"
        }"#,
        )
        .expect("Failed to parse");
        let exp = NewSubscription {
            channel_id: 1,
            frequency: Frequency::Weekly,
            day: Some(Day::Sat),
            time: NaiveTime::from_hms_milli(9, 0, 0, 0),
        };

        assert_eq!(exp, sub);
    }
}
