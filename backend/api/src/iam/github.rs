use github_rs::client::{Executor, Github as GhRsClient};
use github_rs::StatusCode;
use rocket::config::Config;
use rocket_oauth2::hyper_sync_rustls_adapter::HyperSyncRustlsAdapter;
use rocket_oauth2::Adapter;
use rocket_oauth2::{OAuthConfig, TokenRequest};
use serde_json::Value;

use super::*;

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
