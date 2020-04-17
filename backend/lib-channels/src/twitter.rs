use super::channel::*;

use chrono::{DateTime, Utc};
use egg_mode::search::{self, ResultType};
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
        let search = Runtime::new()
            .unwrap()
            .block_on(
                search::search(name.0.to_string())
                    .result_type(ResultType::Recent)
                    .call(&self.token),
            )
            .unwrap();
        for tweet in &search.statuses {
            println!(
                "(@{}) {}",
                tweet.user.as_ref().unwrap().screen_name,
                tweet.text
            );
        }
        Ok(vec![])
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
