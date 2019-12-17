use lib_db as db;

use super::common::*;
use chrono::naive::NaiveTime;
use db::{ChannelType, Day, Frequency};
use rocket::Rocket;
use rocket_contrib::json::{Json, JsonValue};

pub fn mount(rocket: Rocket) -> Rocket {
    rocket.mount("/subscriptions", routes![add])
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Subscription {
    #[serde(rename = "channelName")]
    channel_name: String,
    r#type: ChannelType,
    frequency: Frequency,
    day: Option<Day>,
    time: NaiveTime,
}

impl Into<JsonResponse> for Subscription {
    fn into(self) -> JsonResponse {
        match serde_json::to_value(self) {
            Ok(v) => JsonResponse::Ok(JsonValue(v)),
            Err(_) => JsonResponse::InternalServerError, // todo log
        }
    }
}

impl Subscription {
    fn from_db(sub: db::Subscription, chan: db::Channel) -> Subscription {
        Subscription {
            channel_name: chan.name,
            r#type: chan.type_,
            frequency: sub.frequency,
            day: sub.day,
            time: sub.time,
        }
    }
}

#[post("/add", data = "<new_subscription>")]
fn add(
    session: Protected,
    db: DigesterDbConn,
    new_subscription: Json<Subscription>,
) -> JsonResponse {
    let identity = match db::identities_find_by_user_id(&db, session.0.user_id) {
        Ok(identity) => identity,
        Err(_) => return JsonResponse::InternalServerError,
    };

    let valid_subscription = match validate(new_subscription.0) {
        Ok(valid) => valid,
        Err(msg) => return JsonResponse::BadRequest(msg),
    };

    let channel = match insert_channel_if_not_exists(&db, &valid_subscription) {
        Ok(c) => c,
        Err(_) => return JsonResponse::InternalServerError, // todo log
    };

    match insert_subscription(&db, valid_subscription, &channel, &identity) {
        Ok(sub) => Subscription::from_db(sub, channel).into(),
        Err(_) => JsonResponse::InternalServerError,
    }
}

fn validate(sub: Subscription) -> Result<Subscription, String> {
    // todo validate github repo
    match sub.r#type {
        ChannelType::GithubRelease => {
            if sub.channel_name.contains('/') {
                Ok(sub)
            } else {
                Err("Invalid repository name".to_owned())
            }
        }
    }
}

fn insert_channel_if_not_exists(
    conn: &DigesterDbConn,
    sub: &Subscription,
) -> Result<db::Channel, String> {
    let new_channel = db::NewChannel {
        name: sub.channel_name.clone(),
        type_: sub.r#type,
    };
    db::channels_insert_if_not_exists(&conn.0, new_channel)
}

fn insert_subscription(
    conn: &DigesterDbConn,
    sub: Subscription,
    chan: &db::Channel,
    identity: &db::Identity,
) -> Result<db::Subscription, String> {
    let new_subscription = db::NewSubscription {
        email: identity.email.clone(),
        channel_id: chan.id,
        frequency: sub.frequency,
        day: sub.day,
        time: sub.time,
    };
    db::subscriptions_insert(&conn.0, new_subscription)
}
