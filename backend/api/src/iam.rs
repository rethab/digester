use github_rs::client::{Executor, Github as GhRsClient};
use github_rs::StatusCode;
use rocket::config::Config;
use rocket_contrib::databases::redis::Connection as RedisConnection;
use rocket_oauth2::hyper_sync_rustls_adapter::HyperSyncRustlsAdapter;
use rocket_oauth2::Adapter;
use rocket_oauth2::{OAuthConfig, TokenRequest};
use serde_json::Value;

use diesel::pg::PgConnection;

use super::cache;
use lib_db as db;

use uuid::Uuid;

pub struct User {
    id: i32,
    username: String,
    pub first_login: bool,
}

impl User {
    fn from_db(user: db::User, identity: db::Identity, first_login: bool) -> User {
        User {
            id: user.id,
            username: identity.username,
            first_login,
        }
    }
}

pub struct ProviderUserInfo {
    provider: &'static str,
    pid: String,
    email: String,
    username: String,
}

#[derive(Clone)]
pub struct Session {
    pub id: Uuid,
    pub user_id: i32,
    pub username: String,
}

impl Session {
    pub fn generate(user_id: i32, username: String) -> Session {
        Session {
            id: Uuid::new_v4(),
            user_id,
            username,
        }
    }

    fn from_data(id: Uuid, data: cache::SessionData) -> Session {
        Session {
            id,
            user_id: data.user_id,
            username: data.username,
        }
    }
}

pub struct Github {
    oauth_config: OAuthConfig,
}

impl Github {
    const IDENTIFIER: &'static str = "github";

    pub fn from_rocket_config(config: &Config) -> Result<Github, String> {
        let oauth_config = OAuthConfig::from_config(config, "github")
            .map_err(|err| format!("Failed to read github config from rocket: {:?}", err))?;
        Ok(Github { oauth_config })
    }

    fn fetch_user_info(client: &GhRsClient) -> Result<GitHubUserInfo, String> {
        match client.get().user().execute::<Value>() {
            Ok((_, status, Some(json))) if status == StatusCode::OK => {
                serde_json::from_value::<GitHubUserInfo>(json)
                    .map_err(|err| format!("Failed to parse GitHubUserInfo response: {:?}", err))
            }
            err => Err(format!("Failed to retrieve GitHubUserInfo: {:?}", err)),
        }
    }

    // if a user chooses to set their email private, we don't get it from the /user
    // call above. In that case, we need to reques /user/emails separately. See:
    // https://stackoverflow.com/a/35387123
    fn fetch_email(client: &GhRsClient) -> Result<GitHubUserEmail, String> {
        let pick_email = |emails: Vec<GitHubUserEmail>| {
            let mut emails = emails.into_iter();
            emails
                .find(|email| email.primary)
                .or_else(|| emails.next())
                .ok_or_else(|| "No email found in response".to_owned())
        };
        match client.get().user().emails().execute::<Value>() {
            Ok((_, status, Some(json))) if status == StatusCode::OK => {
                serde_json::from_value::<Vec<GitHubUserEmail>>(json)
                    .map_err(|err| format!("Failed to parse GitHubUserEmail response: {:?}", err))
                    .and_then(pick_email)
            }
            err => Err(format!("Failed to retrieve GitHubUserEmail: {:?}", err)),
        }
    }
}

// the code we get from github when they call us
pub struct AuthorizationCode(pub String);
// the code we get in exchange for the AuthorizationCode
pub struct AccessToken(String);

pub enum AuthenticationError {
    TokenExchangeFailed(String),
    MissingPermissions(String),
    UnknownFailure(String),
}

pub trait IdentityProvider {
    fn exchange_token(&self, code: AuthorizationCode) -> Result<AccessToken, AuthenticationError>;
    fn fetch_user_info(
        &self,
        access_token: AccessToken,
    ) -> Result<ProviderUserInfo, AuthenticationError>;
}

#[derive(Deserialize)]
struct GitHubUserInfo {
    #[serde(rename = "id")]
    pid: i32,
    #[serde(rename = "login")]
    username: String,
    // can be none if user chooses to hide it
    email: Option<String>,
}

#[derive(Deserialize)]
struct GitHubUserEmail {
    email: String,
    primary: bool,
}

impl IdentityProvider for Github {
    fn exchange_token(&self, code: AuthorizationCode) -> Result<AccessToken, AuthenticationError> {
        let hyper = HyperSyncRustlsAdapter {};
        let access_token = hyper
            .exchange_code(&self.oauth_config, TokenRequest::AuthorizationCode(code.0))
            .map(|token_resp| AccessToken(token_resp.access_token().to_owned()))
            .map_err(|err| {
                AuthenticationError::UnknownFailure(format!(
                    "Failed to exchange code for access token: {:?}",
                    err
                ))
            })?;
        Ok(access_token)
    }

    fn fetch_user_info(
        &self,
        access_token: AccessToken,
    ) -> Result<ProviderUserInfo, AuthenticationError> {
        use AuthenticationError::*;

        let client: GhRsClient = GhRsClient::new(access_token.0).map_err(|err| {
            UnknownFailure(format!("Failed to initialize github client: {:?}", err))
        })?;

        match Github::fetch_user_info(&client) {
            Err(msg) => Err(UnknownFailure(msg)),
            Ok(raw) => {
                let email = match raw.email {
                    Some(email) => email,
                    None => Github::fetch_email(&client)
                        .map(|email| email.email)
                        .map_err(|err| {
                            UnknownFailure(format!("Failed to fetch e-mail separately: {:?}", err))
                        })?,
                };
                Ok(ProviderUserInfo {
                    provider: Github::IDENTIFIER,
                    pid: raw.pid.to_string(),
                    email,
                    username: raw.username,
                })
            }
        }
    }
}

pub fn authenticate<P: IdentityProvider>(
    conn: &PgConnection,
    cache: &mut RedisConnection,
    provider: &P,
    code: AuthorizationCode,
) -> Result<(User, Session), AuthenticationError> {
    let access_token = provider.exchange_token(code)?;
    let user_info = provider.fetch_user_info(access_token)?;
    let user = fetch_or_insert_user_in_db(conn, &user_info)
        .map_err(AuthenticationError::UnknownFailure)?;
    let session = create_session(cache, &user).map_err(AuthenticationError::UnknownFailure)?;
    Ok((user, session))
}

pub fn fetch_session(cache: &RedisConnection, session_id: Uuid) -> Result<Option<Session>, String> {
    let id = cache::SessionId(session_id);
    let maybe_data = cache::session_find(cache, id)?;
    Ok(maybe_data.map(|data| Session::from_data(session_id, data)))
}

pub fn logout(cache: &mut RedisConnection, session: Session) -> Result<(), String> {
    cache::session_delete(cache, cache::SessionId(session.id))
}

fn fetch_or_insert_user_in_db(
    conn: &PgConnection,
    user_info: &ProviderUserInfo,
) -> Result<User, String> {
    let maybe_user = db::users_find_by_provider(conn, user_info.provider, &user_info.pid)
        .map_err(|err| format!("error while looking up user in db: {}", err))?;
    match maybe_user {
        Some((user, identity)) => Ok(User::from_db(user, identity, false)),
        None => {
            let new_identity = db::NewUserData {
                provider: user_info.provider.to_owned(),
                pid: user_info.pid.to_owned(),
                email: user_info.email.to_owned(),
                username: user_info.username.to_owned(),
            };
            let (user, identity) = db::users_insert(conn, new_identity)?;
            Ok(User::from_db(user, identity, true))
        }
    }
}

fn create_session(c: &mut RedisConnection, user: &User) -> Result<Session, String> {
    let session = Session::generate(user.id, user.username.clone());
    let data = cache::SessionData {
        user_id: user.id,
        username: user.username.clone(),
    };
    cache::session_store(c, cache::SessionId(session.id), &data)?;
    Ok(session)
}
