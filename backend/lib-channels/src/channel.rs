#![deny(missing_docs)]

use chrono::{DateTime, Utc};

use super::github_release::GithubRelease;
use super::rss::Rss;

/// A channel type is a certain source that we can pull updates
/// from
pub enum ChannelType {
    /// Fetch releases from a specific github repository
    GithubRelease,
    /// Fetch new items from an rss feed
    RssFeed,
}

/// An update is a new thing from a channel. In RSS terminology, this
/// would be an item.
#[derive(Debug)]
pub struct Update {
    /// The title of the update could be the title of a blog post
    /// or the name/version of the new release
    pub title: String,
    /// The url points to some place where the user can read more about
    /// this. For a blog post, this would be a link to the post.
    pub url: String,
    /// The datetime when the update was published in the channel.
    pub published: DateTime<Utc>,
}

/// The failure cases when validating a channel
#[derive(Debug)]
pub enum SearchError {
    /// The channel format is valid, but it doesn't exist (404)
    ChannelNotFound,
    /// Something went wrong. Please try again later
    TechnicalError(String),
}

/// a specific type for channel names that have passed sanitization
/// TODO: make this a generic associated type (problem was to
///       type the factory function in fetcher.rs)
#[derive(Clone)]
pub struct SanitizedName(pub String);

/// Identifies a specific channel where we can pull updates from
#[derive(Debug, PartialEq)]
pub struct ChannelInfo {
    /// human readable description of the channel (eg. "the morning paper")
    pub name: String,
    /// url where we can pull updates from (eg. https://blog.acolyer.org/feed/rss.xml)
    pub url: String,
    /// link to the website about this channel
    /// (eg. https://blog.acolyer.org). something a human would visit.
    pub link: String,
}

/// A channel is a thing where we can pull updates from.
///
/// For example there is a channel type RSS, which allows
/// us to pull items from a blog. In that case, we would
/// have a channel with type Rss and the url of the blog
/// as the name.
///
/// Anther channel could be GithubReleases (as the type)
/// and the name would be the name of a specific repository.
pub trait Channel {
    /// parses the generic name into a channel specific
    /// type, which will be later passed as parameter
    fn sanitize(&self, name: &str) -> Result<SanitizedName, String>;

    /// prepare the user's query for a lookup in the database. this
    /// particularly means trimming off unnecessary things to make
    /// sure a %query% yields the desired results. eg. in case of
    /// urls this can mean to remove the scheme, because the user
    /// might search for https while we stored http, which would then
    /// not match
    ///
    /// The default implementation is to use the regular sanitize
    /// function, because it probably suffices for most cases.
    fn sanitize_for_db_search(&self, query: &str) -> Result<SanitizedName, String> {
        self.sanitize(query)
    }

    /// (Online-) search for channels. For example a user could
    /// search for 'blog.acolyer.org' and we would return channel
    /// infos about the various feeds associated with that website
    fn search(&self, query: SanitizedName) -> Result<Vec<ChannelInfo>, SearchError>;

    /// Fetches updates from the channel. The parameter last_fetched
    /// incates the last time we fetched from this channel. This method
    /// must not return any updates that were published before the
    /// last_fetched. If last_fetched is None, we never fetched from it.
    fn fetch_updates(
        &self,
        name: &str,
        url: &str,
        last_fetched: Option<DateTime<Utc>>,
    ) -> Result<Vec<Update>, String>;
}

/// Factory function to create the channel based on the channel type.
pub fn factory(channel_type: ChannelType, github_release: &GithubRelease) -> &dyn Channel {
    match channel_type {
        ChannelType::GithubRelease => github_release,
        ChannelType::RssFeed => &Rss {},
    }
}
