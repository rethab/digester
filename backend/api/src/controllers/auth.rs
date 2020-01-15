use super::super::iam;

use iam::facebook::Facebook;
use iam::github::Github;
use iam::IdentityProvider;

use super::common::*;

use rocket::http::{Cookie, Cookies, SameSite};
use rocket::{self, Rocket, State};

use rocket_contrib::json::Json;

use uuid::Uuid;

pub fn mount(rocket: Rocket) -> Rocket {
    rocket.mount(
        "/auth",
        routes![me, logout, facebook_oauth_exchange, github_oauth_exchange],
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
        .unwrap_or_default();
    Cookie::build(SESSION_ID, value)
        .same_site(SameSite::Strict)
        .secure(true) // only send via https
        .path("/")
        .http_only(true) // don't give client access, helps a bit with XSS
        .max_age(iam::Session::lifetime())
        .finish()
}

// the request we get from vue-authenticate containing the
// 'code', which we can exchange for an access token at the
// identity provider
#[derive(Deserialize, Debug, PartialEq)]
struct CodeRequest {
    code: String,
    #[serde(rename = "clientId")]
    client_id: String,
    #[serde(rename = "redirectUri")]
    redirect_uri: String,
}

#[post("/github", data = "<oauth_data>")]
fn github_oauth_exchange(
    db: DigesterDbConn,
    redis: Redis,
    cookies: Cookies,
    oauth_data: Json<CodeRequest>,
    provider: State<Github>,
    _r: RateLimited,
) -> JsonResponse {
    let code = iam::AuthorizationCode(oauth_data.0.code);
    oauth_exchange::<Github>(db, redis, cookies, provider, code)
}

#[post("/facebook", data = "<oauth_data>")]
fn facebook_oauth_exchange(
    db: DigesterDbConn,
    redis: Redis,
    cookies: Cookies,
    oauth_data: Json<CodeRequest>,
    provider: State<Facebook>,
    _r: RateLimited,
) -> JsonResponse {
    let code = iam::AuthorizationCode(oauth_data.0.code);
    oauth_exchange::<Facebook>(db, redis, cookies, provider, code)
}

fn oauth_exchange<P: IdentityProvider + Sync + Send>(
    db: DigesterDbConn,
    mut redis: Redis,
    mut cookies: Cookies,
    provider: State<P>,
    code: iam::AuthorizationCode,
) -> JsonResponse {
    use iam::AuthenticationError;
    match iam::authenticate::<P>(&db.0, &mut redis.0, &provider, code) {
        Ok((user, session)) => {
            let cookie = create_session_cookie(Some(session.id));
            cookies.add(cookie);
            JsonResponse::Ok(json!({
                "username": session.username,
                // on the first login, we're trying to automatically set the timezone.
                "first_login": user.first_login,
                // we need to pass an access token back, because vue-authenticate looks at the
                // response and wants to extract the access token and store it in some storage.
                // After that, they call isAuthenticated(), which throws an error if nothing is
                // in the storage. Therefore, we just set a dummy value.
                // See here: https://github.com/dgrubelic/vue-authenticate/blob/3ace24c36580d81fe4a1e748a28b997df2bbb706/src/authenticate.js#L215
                "access_token": "dummy"
            }))
        }
        Err(AuthenticationError::MissingPermissions(msg)) => {
            println!("Missing permissions: {}", msg);
            JsonResponse::BadRequest("Not all necessary scopes granted".into())
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
