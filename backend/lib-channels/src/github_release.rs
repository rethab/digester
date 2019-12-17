use super::channel::*;

use chrono::{DateTime, Utc};

pub struct GithubRelease;

impl Channel for GithubRelease {
    fn validate(&self, _name: &str) -> Result<String, String> {
        unimplemented!()
    }
    fn fetch_updates(
        &self,
        _name: &str,
        _last_fetched: Option<DateTime<Utc>>,
    ) -> Result<Vec<Update>, String> {
        unimplemented!()
    }
}
