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

use std::io::Read;

pub struct User {}

pub struct UserInfo {
    id: String,
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
    fn identifier(&self) -> &str;
    fn exchange_token(&self, code: AuthorizationCode) -> Result<AccessToken, AuthenticationError>;
    fn fetch_user_info(&self, access_token: AccessToken) -> Result<UserInfo, AuthenticationError>;
}

#[derive(serde::Deserialize)]
struct GitHubUserInfo {
    id: String,
}

impl GitHubUserInfo {
    fn user_info(self) -> UserInfo {
        UserInfo { id: self.id }
    }
}

impl IdentityProvider for Github {
    fn identifier(&self) -> &'static str {
        "github"
    }

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

    fn fetch_user_info(&self, access_token: AccessToken) -> Result<UserInfo, AuthenticationError> {
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
    provider: &P,
    code: AuthorizationCode,
) -> Result<Session, AuthenticationError> {
    let access_token = provider.exchange_token(code)?;
    let user_info = provider.fetch_user_info(access_token)?;
    let user = fetch_or_insert_user_in_db(&user_info, provider.identifier())
        .map_err(AuthenticationError::UnknownFailure)?;
    let session = create_session(&user).map_err(AuthenticationError::UnknownFailure)?;
    Ok(session)
}

fn fetch_or_insert_user_in_db(user_info: &UserInfo, provider: &str) -> Result<User, String> {
    unimplemented!()
}

fn create_session(user: &User) -> Result<Session, String> {
    unimplemented!()
}
