use super::channel::*;

use egg_mode::tweet::{self, Tweet};
use egg_mode::user::{self, UserID};
use egg_mode::Token;
use std::collections::HashMap;
use tokio::runtime::Runtime;

pub struct Twitter {
    token: Token,
}

impl Twitter {
    pub fn new(
        api_key: &str,
        api_secret_key: &str,
        access_token: &str,
        access_token_secret: &str,
    ) -> Result<Twitter, String> {
        let con_token = egg_mode::KeyPair::new(api_key.to_owned(), api_secret_key.to_owned());
        let access_token =
            egg_mode::KeyPair::new(access_token.to_owned(), access_token_secret.to_owned());
        let token = egg_mode::Token::Access {
            consumer: con_token,
            access: access_token,
        };
        Ok(Twitter { token })
    }

    /// given a list of tweet ids, returns the ones that no longer exist and therefore
    /// have to be deleted. we need to do this in order to comply with twitter's visibily
    /// policy, which says that if a tweet is deleted, it must be deleted in our system
    /// within 24h
    pub fn find_to_delete(&self, ids: Vec<u64>) -> Result<Vec<u64>, String> {
        let mut rt = Runtime::new()
            .map_err(|err| format!("Failed to initialize tokio runtime: {:?}", err))?;
        rt.block_on(self.find_to_delete0(ids))
    }

    async fn find_to_delete0(&self, ids: Vec<u64>) -> Result<Vec<u64>, String> {
        tweet::lookup_map(ids.clone(), &self.token)
            .await
            .map(|resp| filter_missing_tweets(resp.response))
            .map_err(|err| format!("Failed to fetch tweets by ids {:?}: {:?}", ids, err))
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

    fn fetch_updates(&self, screen_name: &str) -> Result<Vec<Update>, String> {
        let mut rt = Runtime::new()
            .map_err(|err| format!("Failed to initialize tokio runtime: {:?}", err))?;
        rt.block_on(tweet_search(screen_name.to_owned(), &self.token))
    }
}

async fn tweet_search(screen_name: String, token: &Token) -> Result<Vec<Update>, String> {
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
                    ext_id: Some(tweet.id.to_string()),
                    title: tweet.text.clone(),
                    url: format!("https://twitter.com/{}/status/{}", screen_name, tweet.id),
                    published: tweet.created_at,
                };
                updates.push(update);
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

fn filter_missing_tweets(ts: HashMap<u64, Option<Tweet>>) -> Vec<u64> {
    let mut missing = Vec::new();
    for (k, v) in ts.iter() {
        if v.is_none() {
            missing.push(*k);
        }
    }
    missing
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Utc;
    use egg_mode::tweet::TweetEntities;
    use std::collections::hash_map::RandomState;
    use std::collections::HashSet;
    use std::iter::FromIterator;

    #[test]
    fn return_only_ids_with_none() {
        let filtered = filter_missing_tweets(HashMap::from_iter(vec![
            (1, None),
            (2, Some(mk_tweet())),
            (3, None),
            (4, Some(mk_tweet())),
            (5, None),
        ]));
        let missing: HashSet<u64, RandomState> = HashSet::from_iter(vec![1, 3, 5]);
        assert_eq!(missing, HashSet::from_iter(filtered))
    }

    fn mk_tweet() -> Tweet {
        Tweet {
            coordinates: None,
            created_at: Utc::now(),
            current_user_retweet: None,
            display_text_range: None,
            entities: TweetEntities {
                hashtags: Vec::new(),
                symbols: Vec::new(),
                urls: Vec::new(),
                user_mentions: Vec::new(),
                media: None,
            },
            extended_entities: None,
            favorite_count: 1,
            favorited: None,
            filter_level: None,
            id: 1234,
            in_reply_to_user_id: None,
            in_reply_to_screen_name: None,
            in_reply_to_status_id: None,
            lang: None,
            place: None,
            possibly_sensitive: None,
            quoted_status_id: None,
            quoted_status: None,
            retweet_count: 1,
            retweeted: None,
            retweeted_status: None,
            source: None,
            text: "foo".into(),
            truncated: false,
            user: None,
            withheld_copyright: false,
            withheld_in_countries: None,
            withheld_scope: None,
        }
    }
}
