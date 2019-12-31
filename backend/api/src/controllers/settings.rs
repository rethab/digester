use super::common::*;
use chrono_tz::Tz;
use lib_db as db;
use rocket::Rocket;
use rocket_contrib::json::{Json, JsonValue};

pub fn mount(rocket: Rocket) -> Rocket {
    rocket.mount("/settings", routes![get, update])
}

#[derive(Deserialize, Debug, PartialEq)]
struct UpdatedSettings {
    timezone: Tz,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
struct Settings {
    timezone: Option<Tz>,
}

impl Into<JsonResponse> for Settings {
    fn into(self) -> JsonResponse {
        match serde_json::to_value(self.clone()) {
            Ok(v) => JsonResponse::Ok(JsonValue(v)),
            Err(err) => {
                eprintln!("Failed to serialize Settings {:?}: {:?}", self, err);
                JsonResponse::InternalServerError
            }
        }
    }
}

#[get("/")]
fn get(session: Protected, db: DigesterDbConn) -> JsonResponse {
    let user_id = session.0.user_id;
    match db::users_find_by_id(&db.0, user_id) {
        Ok(user) => {
            let settings = Settings {
                timezone: user.timezone.map(|tz| tz.0),
            };
            settings.into()
        }
        Err(err) => {
            eprintln!("Failed to fetch timezone for user {}: {:?}", user_id, err);
            JsonResponse::InternalServerError
        }
    }
}

#[post("/", data = "<updated_settings>")]
fn update(
    session: Protected,
    db: DigesterDbConn,
    updated_settings: Json<UpdatedSettings>,
) -> JsonResponse {
    let user_id = session.0.user_id;
    let new_tz = updated_settings.0.timezone;
    let user = match db::users_find_by_id(&db.0, user_id) {
        Ok(user) => user,
        Err(err) => {
            eprintln!("Failed to load user with id {}: {:?}", user_id, err);
            return JsonResponse::InternalServerError;
        }
    };

    // avoid unnecessary updates (invalidating digests could be painful)
    if user.timezone.as_ref().map(|tz| tz.0).contains(&new_tz) {
        return JsonResponse::Ok(json!({}));
    }

    match db::users_update_timezone(&db.0, user_id, new_tz) {
        Ok(()) => {
            println!("Updated timezone of user {} to {:?}", user_id, new_tz);
            if let Err(err) = db::digests_remove_unsent_for_user(&db.0, &user) {
                eprintln!(
                    "Failed to invalidate digests after timezone update for user {}: {:?}",
                    user_id, err
                );
            }
            JsonResponse::Ok(json!({}))
        }
        Err(err) => {
            eprintln!(
                "Failed to update timezone of user {} to {:?}: {:?}",
                user_id, new_tz, err
            );
            JsonResponse::InternalServerError
        }
    }
}
