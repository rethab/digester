use super::channel::*;

use chrono::{DateTime, Utc};
use egg_mode::tweet;
use egg_mode::user::{self, UserID};
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
        screen_name: &str,
        last_fetched: Option<DateTime<Utc>>,
    ) -> Result<Vec<Update>, String> {
        let mut rt = Runtime::new()
            .map_err(|err| format!("Failed to initialize tokio runtime: {:?}", err))?;
        rt.block_on(tweet_search(
            screen_name.to_owned(),
            last_fetched,
            &self.token,
        ))
    }
}

async fn tweet_search(
    screen_name: String,
    last_fetched: Option<DateTime<Utc>>,
    token: &Token,
) -> Result<Vec<Update>, String> {
    let result = tweet::user_timeline(
        UserID::ScreenName(screen_name.clone().into()),
        false, /* replies */
        false, /* retweets */
        token,
    )
    .with_page_size(100) // page size is applied before filtering replies and retweets
    .start()
    .await;

    match result {
        Err(err) => Err(format!(
            "Failed to fetch tweets for {}: {:?}",
            screen_name, err
        )),
        Ok((_, feed)) => {
            let mut updates = Vec::new();
            for tweet in &*feed {
                let update = Update {
                    title: tweet.text.clone(),
                    url: format!("https://twitter.com/{}/{}", screen_name, tweet.id),
                    published: tweet.created_at,
                };
                if !update.is_old(last_fetched) {
                    updates.push(update);
                }
            }
            Ok(updates)
        }
    }
}

async fn user_search(query: SanitizedName, token: &Token) -> Result<Vec<ChannelInfo>, SearchError> {
    let results = user::search(query.0.to_string(), token)
        .with_page_size(10)
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
                if !user.protected {
                    channel_infos.push(ChannelInfo {
                        ext_id: user.screen_name.clone(),
                        name: user.name,
                        link: format!("https://twitter.com/{}", user.screen_name),
                        verified: user.verified,
                    })
                }
            }
            Ok(channel_infos)
        }
    }
}
