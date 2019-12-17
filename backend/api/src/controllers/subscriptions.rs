use lib_channels::github_release::GithubRelease;
use lib_channels::{Channel, ValidationError};
use lib_db as db;

use super::common::*;
use chrono::naive::NaiveTime;
use db::{ChannelType, Day, Frequency};
use rocket::{Rocket, State};
use rocket_contrib::json::{Json, JsonValue};

pub fn mount(rocket: Rocket) -> Rocket {
    rocket.mount("/subscriptions", routes![list, add])
}

pub struct GithubApiToken(pub String);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Subscription {
    #[serde(rename = "channelName")]
    channel_name: String,
    channel_type: ChannelType,
    frequency: Frequency,
    day: Option<Day>,
    time: NaiveTime,
}

impl Subscription {
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
        match serde_json::to_value(self) {
            Ok(v) => JsonResponse::Ok(JsonValue(v)),
            Err(_) => JsonResponse::InternalServerError, // todo log
        }
    }
}

impl Into<JsonResponse> for Vec<Subscription> {
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
            channel_type: chan.channel_type,
            frequency: sub.frequency,
            day: sub.day,
            time: sub.time,
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

#[post("/add", data = "<new_subscription>")]
fn add(
    session: Protected,
    db: DigesterDbConn,
    github_api_token: State<GithubApiToken>,
    new_subscription: Json<Subscription>,
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
        Err(_) => return JsonResponse::InternalServerError, // todo log
    };

    match insert_subscription(&db, valid_subscription, &channel, &identity) {
        Ok(sub) => Subscription::from_db(sub, channel).into(),
        Err(_) => JsonResponse::InternalServerError,
    }
}

fn validate(sub: Subscription, gh_token: &GithubApiToken) -> Result<Subscription, String> {
    match sub.channel_type {
        ChannelType::GithubRelease => {
            // todo: if we already know this repo, no need to call github
            let github: GithubRelease =
                GithubRelease::new(&gh_token.0).map_err(|_| "Unknown problem".to_owned())?;
            github
                .validate(&sub.channel_name)
                .map(|repo_name| sub.with_name(repo_name))
                .map_err(|err| match err {
                    ValidationError::ChannelInvalid(msg) => msg,
                    ValidationError::ChannelNotFound => "Repository does not exist".into(),
                    ValidationError::TechnicalError => "Unknown error".into(),
                })
        }
    }
}

fn insert_channel_if_not_exists(
    conn: &DigesterDbConn,
    sub: &Subscription,
) -> Result<db::Channel, String> {
    let new_channel = db::NewChannel {
        name: sub.channel_name.clone(),
        channel_type: sub.channel_type,
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
        user_id: identity.user_id,
        frequency: sub.frequency,
        day: sub.day,
        time: sub.time,
    };
    db::subscriptions_insert(&conn.0, new_subscription)
}
