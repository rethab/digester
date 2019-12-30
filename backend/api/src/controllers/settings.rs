use super::common::*;
use chrono_tz::Tz;
use lib_db as db;
use rocket::Rocket;
use rocket_contrib::json::Json;

pub fn mount(rocket: Rocket) -> Rocket {
    rocket.mount("/settings", routes![update])
}

#[derive(Deserialize, Debug, PartialEq)]
struct UpdatedSettings {
    timezone: Tz,
}

#[post("/", data = "<updated_settings>")]
fn update(
    session: Protected,
    db: DigesterDbConn,
    updated_settings: Json<UpdatedSettings>,
) -> JsonResponse {
    let user_id = session.0.user_id;
    let new_tz = updated_settings.0.timezone;
    match db::users_update_timezone(&db.0, user_id, new_tz) {
        Ok(()) => {
            println!("Updated timezone of user {} to {:?}", user_id, new_tz);
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
