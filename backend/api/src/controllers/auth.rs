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
        routes![
            me,
            logout,
            delete_challenge,
            delete_account,
            facebook_oauth_exchange,
            github_oauth_exchange
        ],
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
                "userId": session.user_id,
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
            eprintln!("Missing permissions: {}", msg);
            JsonResponse::BadRequest("missing_permissions".into())
        }
        Err(AuthenticationError::UnknownFailure(msg)) => {
            eprintln!("Unknown auth failure: {}", msg);
            JsonResponse::InternalServerError
        }
        Err(AuthenticationError::TokenExchangeFailed(msg)) => {
            eprintln!("token exchange failure: {}", msg);
            JsonResponse::InternalServerError
        }
    }
}

#[get("/me")]
fn me(session: Protected) -> JsonResponse {
    JsonResponse::Ok(json! ({
        "username": session.0.username,
        "userId": session.0.user_id,
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

#[get("/delete_challenge")]
fn delete_challenge(session: Protected, mut redis: Redis, _r: RateLimited) -> JsonResponse {
    match iam::create_delete_challenge(&mut redis, session.0.user_id) {
        Ok(challenge) => JsonResponse::Ok(json!({ "challenge": challenge })),
        Err(err) => {
            eprintln!(
                "Failed to create challenge for {}: {:?}",
                session.0.user_id, err
            );
            JsonResponse::InternalServerError
        }
    }
}

/// When a user wants to delete their account, we
/// send them a delete challenge, which is a random
/// string they need to repeat back. This 'answer'
/// is the challenge response. If this matches the
/// original challenge we sent them (we keep it in
/// our cache), the account may be deleted.
///
/// The idea of using this challenge is that the user
/// needs an interaction, so deleting the account
/// cannot (as) easily be automated as just sending
/// a delete request. However if we were fully
/// vulnerable to XSS in the sense that the attacker
/// could execute two request, the attack might as well
/// first fetch the challenge and then respond with it.
#[derive(Deserialize)]
struct DeleteChallengeResponse {
    response: String,
}

#[delete("/me", data = "<challenge_response>")]
fn delete_account(
    challenge_response: Json<DeleteChallengeResponse>,
    session: Protected,
    mut redis: Redis,
    mut cookies: Cookies,
    db: DigesterDbConn,
    _r: RateLimited,
) -> JsonResponse {
    use iam::DeleteError::*;
    match iam::delete_account(
        &mut redis,
        &db,
        session.0.user_id,
        &challenge_response.0.response,
    ) {
        Ok(()) => {
            if let Err(err) = iam::logout(&mut redis, session.0) {
                eprintln!("Failed to logout after deleting account: {}", err);
            }
            let cookie = create_session_cookie(None);
            cookies.remove(cookie);
            JsonResponse::Ok(json!({}))
        }
        Err(InvalidChallengeResponse) => {
            JsonResponse::BadRequest("Invalid challenge response".into())
        }
        Err(MissingChallenge) => JsonResponse::BadRequest("Expired challenge response".into()),
        Err(Unknown(err)) => {
            eprintln!(
                "Failed to create challenge for {}: {:?}",
                session.0.user_id, err
            );
            JsonResponse::InternalServerError
        }
    }
}
