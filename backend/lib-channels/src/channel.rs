use chrono::{DateTime, Utc};

pub enum ChannelType {
    GithubRelease,
    Rss,
}

pub struct Update {
    pub title: String,
    pub url: String,
    pub published: DateTime<Utc>,
}

pub trait Channel {
    fn validate(&self, name: &str) -> Result<String, String>;
    fn fetch_updates(
        &self,
        name: &str,
        last_fetched: Option<DateTime<Utc>>,
    ) -> Result<Vec<Update>, String>;
}
