use super::super::iam;

use super::common::*;

use rocket::http::{Cookie, Cookies};
use rocket::{self, Rocket, State};

use rocket_contrib::json::Json;
use serde::Deserialize;

use time::Duration;
use uuid::Uuid;

pub fn mount(rocket: Rocket) -> Rocket {
    rocket.mount(
        "/auth",
        routes![me, me_unauthenticated, logout, github_oauth_exchange],
    )
}

// creates the session cookie. a None value creates a removal cookie
fn create_session_cookie(maybe_id: Option<Uuid>) -> Cookie<'static> {
    let value = maybe_id
        .map(|id| {
            id.to_simple()
                .encode_lower(&mut Uuid::encode_buffer())
                .to_owned()
        })
        .unwrap_or(String::new());
    // todo review cookie settings
    Cookie::build("SESSION_ID", value)
        .domain("localhost")
        .secure(false)
        .path("/")
        .http_only(false)
        .max_age(Duration::days(1))
        .finish()
}

#[derive(Deserialize, Debug, PartialEq)]
struct BlaBla {
    code: String,
    #[serde(rename = "clientId")]
    client_id: String,
    #[serde(rename = "redirectUri")]
    redirect_uri: String,
}

// todo how to prevent malicious users from calling this? (and us essentially being a github proxy)
#[post("/github", data = "<oauth_data>")]
fn github_oauth_exchange(
    db: DigesterDbConn,
    mut redis: Redis,
    mut cookies: Cookies,
    oauth_data: Json<BlaBla>,
    provider: State<iam::Github>,
) -> JsonResponse {
    use iam::AuthenticationError;
    let code = iam::AuthorizationCode(oauth_data.0.code);
    match iam::authenticate::<iam::Github>(&db.0, &mut redis.0, &provider, code) {
        Ok(session) => {
            let cookie = create_session_cookie(Some(session.id.clone()));
            cookies.add(cookie);
            JsonResponse::Ok(json!({
                "username": session.username,
            }))
        }
        Err(AuthenticationError::UnknownFailure(msg)) => {
            println!("Unknown auth failure: {}", msg);
            JsonResponse::InternalServerError
        }
        Err(AuthenticationError::TokenExchangeFailed(msg)) => {
            println!("token exchange failure: {}", msg);
            JsonResponse::InternalServerError
        }
    }
}

#[get("/me")]
fn me(session: Protected) -> JsonResponse {
    JsonResponse::Ok(json! ({
        "username": session.0.username
    }))
}

#[get("/me", rank = 2)]
fn me_unauthenticated() -> JsonResponse {
    JsonResponse::Unauthorized
}

#[post("/logout")]
fn logout(
    maybe_session: Option<Protected>,
    mut redis: Redis,
    mut cookies: Cookies,
) -> JsonResponse {
    match maybe_session {
        Some(session) => {
            println!("Destroying session");
            match iam::logout(&mut redis.0, session.0) {
                Ok(()) => (),
                Err(_) => return JsonResponse::InternalServerError,
            }
        }
        None => {
            println!("No session to destroy");
        }
    }
    let cookie = create_session_cookie(None);
    cookies.remove(cookie);
    JsonResponse::Ok(json!({}))
}
