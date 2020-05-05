use lib_db as db;

use super::common::*;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use db::ChannelType;
use rocket::Rocket;
use rocket_contrib::json::JsonValue;

pub fn mount(rocket: Rocket) -> Rocket {
    rocket.mount("/updates", routes![list])
}

#[derive(Serialize, Clone, Debug, PartialEq)]
struct Update {
    #[serde(rename = "channelName")]
    channel_name: String,
    #[serde(rename = "channelType")]
    channel_type: ChannelType,
    #[serde(rename = "channelLink")]
    channel_link: String,
    title: String,
    url: String,
    // in the user's timezone
    published: DateTime<Tz>,
}

impl Into<JsonResponse> for Update {
    fn into(self) -> JsonResponse {
        match serde_json::to_value(self.clone()) {
            Ok(v) => JsonResponse::Ok(JsonValue(v)),
            Err(err) => {
                eprintln!(
                    "Failed to convert Update {:?} into JsonResponse: {:?}",
                    self, err
                );
                JsonResponse::InternalServerError
            }
        }
    }
}

impl Into<JsonResponse> for Vec<Update> {
    fn into(self) -> JsonResponse {
        match serde_json::to_value(self) {
            Ok(v) => JsonResponse::Ok(JsonValue(v)),
            Err(err) => {
                eprintln!("Failed to convert Vec<Update> into JsonResponse: {:?}", err);
                JsonResponse::InternalServerError
            }
        }
    }
}

impl Update {
    fn from_db(update: db::Update, chan: db::Channel, user_tz: Tz) -> Update {
        Update {
            channel_name: chan.name.clone(),
            channel_type: chan.channel_type,
            channel_link: chan.link,
            title: update.title,
            url: update.url,
            published: utc_to_tz(update.published, user_tz),
        }
    }
}

fn utc_to_tz(datetime: DateTime<Utc>, tz: Tz) -> DateTime<Tz> {
    datetime.with_timezone(&tz)
}

#[get("/?<offset>&<limit>")]
fn list(session: Protected, db: DigesterDbConn, offset: u32, limit: u32) -> JsonResponse {
    let user = match db::users_find_by_id(&db, session.0.user_id.into()) {
        Err(err) => {
            eprintln!(
                "Failed to fetch user by id {}: {:?}",
                session.0.user_id, err
            );
            return JsonResponse::InternalServerError;
        }
        Ok(user) => user,
    };

    let timezone = user.timezone.map(|tz| tz.0).unwrap_or_else(|| Tz::UTC);
    match db::updates_find_by_user_id(&db, user.id, offset, limit) {
        Err(err) => {
            eprintln!(
                "Failed to find updates for user {}. offset={}, limit={}. Err={:?}",
                user.id, offset, limit, err
            );
            JsonResponse::InternalServerError
        }
        Ok(subs) => subs
            .into_iter()
            .map(|(update, chan)| Update::from_db(update, chan, timezone))
            .collect::<Vec<Update>>()
            .into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::offset::TimeZone;
    use chrono_tz::Europe::Amsterdam;

    #[test]
    fn timezone_conversion() {
        let utc = Utc.ymd(2020, 1, 14).and_hms(9, 50, 0);
        let expected = Amsterdam.ymd(2020, 1, 14).and_hms(10, 50, 0);
        assert_eq!(expected, utc_to_tz(utc, Amsterdam))
    }
}
