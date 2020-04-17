use super::channel::*;

use chrono::{DateTime, Utc};
use egg_mode::user;
use egg_mode::Token;
use tokio::runtime::Runtime;

pub struct Twitter {
    token: Token,
}

impl Twitter {
    pub fn new(_api_token: &str) -> Result<Twitter, String> {
        let con_token = egg_mode::KeyPair::new(
            "jRnAWOvekQxdISOza67q8wQ3h",
            "YveBg4qocPHfR2MqPGbdneW3vn8lgqzPn8BXPXmmfHUzefDxgU",
        );
        let access_token = egg_mode::KeyPair::new(
            "1216769639667130374-0tluDWdgmuzoWbhYmCHF8tFkzcU7Re",
            "LRUXb2p5ot1imqQ67sb8IBxcm8L4m8mLIz2oFnGBZZTD7",
        );
        let token = egg_mode::Token::Access {
            consumer: con_token,
            access: access_token,
        };
        Ok(Twitter { token })
    }
}

impl Channel for Twitter {
    fn sanitize(&self, name: &str) -> Result<SanitizedName, String> {
        Ok(SanitizedName(name.to_owned()))
    }

    fn search(&self, name: SanitizedName) -> Result<Vec<ChannelInfo>, SearchError> {
        let mut rt = Runtime::new().map_err(|err| {
            SearchError::TechnicalError(format!("Failed to initialize tokio runtime: {:?}", err))
        })?;
        rt.block_on(user_search(name, &self.token))
    }

    fn fetch_updates(
        &self,
        _name: &str,
        _url: &str,
        _last_fetched: Option<DateTime<Utc>>,
    ) -> Result<Vec<Update>, String> {
        unimplemented!()
    }
}

async fn user_search(query: SanitizedName, token: &Token) -> Result<Vec<ChannelInfo>, SearchError> {
    let results = user::search(query.0.to_string(), token)
        .with_page_size(20)
        .call()
        .await;

    match results {
        Err(err) => Err(SearchError::TechnicalError(format!(
            "Failed to call twitter: {:?}",
            err
        ))),
        Ok(users) => {
            let mut channel_infos = Vec::with_capacity(users.len());
            for user in users.response {
                channel_infos.push(ChannelInfo {
                    name: user.name,
                    url: format!("https://twitter.com/{}", user.screen_name),
                    link: format!("https://twitter.com/{}", user.screen_name),
                })
            }
            Ok(channel_infos)
        }
    }
}
