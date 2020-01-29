use super::*;

use reqwest::blocking::Client;
use rocket::config::Config;
use rocket_oauth2::hyper_sync_rustls_adapter::HyperSyncRustlsAdapter;
use rocket_oauth2::{Adapter, OAuthConfig, TokenRequest};

pub struct Facebook {
    oauth_config: OAuthConfig,
}

impl Facebook {
    const IDENTIFIER: &'static str = "facebook";

    pub fn from_rocket_config(config: &Config) -> Result<Facebook, String> {
        let oauth_config = OAuthConfig::from_config(config, "facebook")
            .map_err(|err| format!("Failed to read facebook config from rocket: {:?}", err))?;
        Ok(Facebook { oauth_config })
    }
}

#[derive(Deserialize)]
struct FacebookMeResponse {
    id: String,
    email: Option<String>,
    name: Option<String>,
}

impl IdentityProvider for Facebook {
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
        let url = format!(
            "https://graph.facebook.com/me?access_token={}&fields=email,name",
            access_token.0
        );
        let resp = match Client::new().get(&url).send() {
            Ok(resp) if resp.status().is_success() => resp,
            Ok(resp) => {
                return Err(UnknownFailure(format!(
                "Non-200 response from facebook's graph api status={}, body={}, access_token={}",
                resp.status(),
                resp.text().unwrap_or_else(|_|"[no body]".into()),
                access_token.0,
            )))
            }
            Err(err) => {
                return Err(UnknownFailure(format!(
                    "Failed to query facebook's graph for access_token {}: {:?}",
                    access_token.0, err,
                )))
            }
        };
        match resp.json::<FacebookMeResponse>() {
            Ok(me) => {
                match me.email {
                    Some(email) => {
                        Ok(ProviderUserInfo {
                            provider: Facebook::IDENTIFIER,
                            pid: me.id,
                            email: email.clone(),
                            // todo should the username become optional
                            username: me.name.unwrap_or_else(|| email),
                        })
                    }
                    None => Err(MissingPermissions(format!(
                        "Field 'email' missing in facebook response for {}",
                        me.id
                    ))),
                }
            }
            Err(err) => Err(UnknownFailure(format!(
                "Failed to parse facebook's response. err:{:?}",
                err
            ))),
        }
    }
}
