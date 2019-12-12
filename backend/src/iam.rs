use hyper::{
    header::{qitem, Accept, Authorization, UserAgent},
    mime::Mime,
    net::HttpsConnector,
    Client,
};
use rocket::config::Config;
use rocket_oauth2::hyper_sync_rustls_adapter::HyperSyncRustlsAdapter;
use rocket_oauth2::Adapter;
use rocket_oauth2::{OAuthConfig, TokenRequest};

use diesel::pg::PgConnection;

use super::db;

use std::io::Read;

pub struct User {
    id: i32,
    username: String,
}

impl User {
    fn from_db(user: db::User, identity: db::Identity) -> User {
        User {
            id: user.id,
            username: identity.username,
        }
    }
}

pub struct ProviderUserInfo {
    provider: &'static str,
    pid: String,
    email: String,
    username: String,
}
pub struct Session {
    pub id: String,
    pub username: String,
}

pub struct Github {
    oauth_config: OAuthConfig,
}

impl Github {
    pub fn from_rocket_config(config: &Config) -> Result<Github, String> {
        let oauth_config = OAuthConfig::from_config(config, "github")
            .map_err(|err| format!("Failed to read github config from rocket: {:?}", err))?;
        Ok(Github { oauth_config })
    }

    const IDENTIFIER: &'static str = "github";
}

// the code we get from github when they call us
pub struct AuthorizationCode(pub String);
// the code we get in exchange for the AuthorizationCode
pub struct AccessToken(String);

pub enum AuthenticationError {
    TokenExchangeFailed(String),
    UnknownFailure(String),
}

pub trait IdentityProvider {
    fn exchange_token(&self, code: AuthorizationCode) -> Result<AccessToken, AuthenticationError>;
    fn fetch_user_info(
        &self,
        access_token: AccessToken,
    ) -> Result<ProviderUserInfo, AuthenticationError>;
}

#[derive(serde::Deserialize)]
struct GitHubUserInfo {
    #[serde(rename = "id")]
    pid: String,
    #[serde(rename = "login")]
    username: String,
    email: String,
}

impl GitHubUserInfo {
    fn user_info(self) -> ProviderUserInfo {
        ProviderUserInfo {
            provider: Github::IDENTIFIER,
            pid: self.pid,
            email: self.email,
            username: self.username,
        }
    }
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
        use AuthenticationError::UnknownFailure;

        let https = HttpsConnector::new(hyper_sync_rustls::TlsClient::new());
        let client = Client::with_connector(https);

        // Use the token to retrieve the user's GitHub account information.
        let mime: Mime = "application/vnd.github.v3+json"
            .parse()
            .expect("parse GitHub MIME type");
        let response = client
            .get("https://api.github.com/user")
            .header(Authorization(format!("token {}", access_token.0)))
            .header(Accept(vec![qitem(mime)]))
            .header(UserAgent("rocket_oauth2 demo application".into()))
            .send()
            .map_err(|err| UnknownFailure(format!("Failed to call github api: {:?}", err)))?;

        if !response.status.is_success() {
            return Err(UnknownFailure(
                format!("got non-success status {}", response.status).into(),
            ));
        }

        let user_info: GitHubUserInfo = serde_json::from_reader(response.take(2 * 1024 * 1024))
            .map_err(|err| {
                UnknownFailure(format!("failed to deserialize github response: {:?}", err))
            })?;

        Ok(user_info.user_info())
    }
}

pub fn authenticate<P: IdentityProvider>(
    conn: &PgConnection,
    provider: &P,
    code: AuthorizationCode,
) -> Result<Session, AuthenticationError> {
    let access_token = provider.exchange_token(code)?;
    let user_info = provider.fetch_user_info(access_token)?;
    let user = fetch_or_insert_user_in_db(conn, &user_info)
        .map_err(AuthenticationError::UnknownFailure)?;
    let session = create_session(&user).map_err(AuthenticationError::UnknownFailure)?;
    Ok(session)
}

fn fetch_or_insert_user_in_db(
    conn: &PgConnection,
    user_info: &ProviderUserInfo,
) -> Result<User, String> {
    let maybe_user = db::users_find_by_provider(conn, user_info.provider, &user_info.pid)
        .map_err(|err| format!("error while looking up user in db: {}", err))?;
    match maybe_user {
        Some((user, identity)) => Ok(User::from_db(user, identity)),
        None => {
            let new_identity = db::NewUserData {
                provider: user_info.provider.to_owned(),
                pid: user_info.pid.to_owned(),
                email: user_info.email.to_owned(),
                username: user_info.username.to_owned(),
            };
            let (user, identity) = db::users_insert(conn, new_identity)?;
            Ok(User::from_db(user, identity))
        }
    }
}

fn create_session(user: &User) -> Result<Session, String> {
    unimplemented!()
}
