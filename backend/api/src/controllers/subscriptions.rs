use lib_channels::rss;
use lib_db as db;

use super::common::*;
use chrono::naive::NaiveTime;
use db::{ChannelType, Day, Frequency};
use rocket::http::RawStr;
use rocket::request::FromFormValue;
use rocket::Rocket;
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
            channel_name: chan.name,
            channel_type: chan.channel_type,
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
    name: String, // human readable name of thing
    // todo make non optional
    link: Option<String>, // link to the website
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
    // fixme _session: Protected,
    db: DigesterDbConn,
    channel_type: SearchChannelType,
    query: &RawStr,
) -> JsonResponse {
    let query = match query.url_decode() {
        Ok(query) => query,
        Err(err) => {
            eprintln!("Failed to decode raw string: '{}': {:?}", query, err);
            return JsonResponse::InternalServerError;
        }
    };

    let sanititzed_url = match rss::SanitizedUrl::parse(&query) {
        Ok(url) => url,
        Err(err) => {
            eprintln!("Query is not a URL: {:?}", err);
            return JsonResponse::BadRequest("not a url".into());
        }
    };

    // we need to search for the sanitized URL as long as we search for exact matches,
    // because otherwise we might store https://bla.bla but the user searches for bla.bla

    // searching w/o scheme means the user can search for http and we find https
    let query = sanititzed_url.to_string_without_scheme();

    let channels = match db::channels_search(&db, channel_type.0, &query) {
        Ok(channels) => channels,
        Err(err) => {
            eprintln!(
                "Failed to search for channel by query '{}': {:?}",
                query, err
            );
            return JsonResponse::InternalServerError;
        }
    };

    println!("Found {} channels in search '{}'", channels.len(), query);

    if !channels.is_empty() {
        return JsonResponse::Ok(json!({
            "channels": channels.into_iter().map(|c| Channel::from_db(c)).collect::<Vec<Channel>>()
        }));
    }

    if channel_type.0 != ChannelType::RssFeed {
        eprintln!(
            "Search is only implemented for RSS feeds: {:?}",
            channel_type.0
        );
        return JsonResponse::InternalServerError;
    }

    let url = sanititzed_url.to_url();
    let feeds = match rss::fetch_feeds(&url) {
        Err(err) => {
            eprintln!("Failed to fetch feeds for url '{}': {:?}", url, err);
            return JsonResponse::InternalServerError;
        }
        Ok(feeds) => feeds,
    };

    if feeds.is_empty() {
        let no_channels: Vec<Channel> = vec![];
        return JsonResponse::Ok(json!({ "channels": no_channels }));
    }

    let channels = feeds
        .iter()
        .map(|f| db::NewChannel {
            channel_type: channel_type.0,
            name: f.url.clone(),
            link: f.link.clone(),
        })
        .collect();

    match db::channels_insert_many(&db, channels) {
        Err(err) => {
            eprintln!("Failed to insert new channels: {:?}", err);
            JsonResponse::InternalServerError
        }
        Ok(channels) => JsonResponse::Ok(json!({
            "channels": channels.into_iter().map(|c| Channel::from_db(c)).collect::<Vec<Channel>>()
        })),
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
            "channelId":"1",
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
