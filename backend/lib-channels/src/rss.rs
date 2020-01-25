use super::channel::*;

use atom_syndication::Error as AtomError;
use atom_syndication::Feed;
use chrono::{DateTime, Utc};
use kuchiki::traits::*;
use reqwest::header::{ToStrError, CONTENT_TYPE};
use reqwest::StatusCode;
use rss::Channel as RssChannel;
use rss::Error as RssError;
use std::fmt::Display;
use std::io::BufReader;
use url::Url;

/// While it is called RSS, this
/// actually also works with Atom
/// feeds
pub struct Rss {}

#[derive(PartialEq, Debug)]
pub struct SanitizedUrl {
    scheme: String,
    host: String,
    path: String,
}

impl SanitizedUrl {
    fn to_string(&self) -> String {
        format!("{}://{}{}", self.scheme, self.host, self.path,)
    }

    pub fn to_string_without_scheme(&self) -> String {
        format!("{}{}", self.host, self.path,)
    }

    pub fn to_url(&self) -> Url {
        // unwrap is safe, because we only construct sanitized urls from valid urls
        Url::parse(&self.to_string()).unwrap()
    }

    fn from_url(url: Url) -> Result<Self, String> {
        let minimum_length = |s: &str| {
            let pieces: Vec<&str> = s.split('.').collect();
            pieces.len() >= 2 && pieces.last().unwrap().len() >= 2
        };

        let valid_scheme = |s: &str| s == "http" || s == "https";

        if url.port().is_some() {
            return Err("cannot have port".to_owned());
        }

        if !valid_scheme(url.scheme()) {
            return Err(format!("invalid scheme: {}", url.scheme()));
        }
        match url.host() {
            Some(url::Host::Domain(d)) if minimum_length(&d) => Ok(Self {
                scheme: url.scheme().into(),
                host: d.into(),
                path: url.path().into(),
            }),
            Some(url::Host::Domain(_)) => Err("missing tld".to_owned()),
            Some(_ip) => Err("cannot be ip".to_owned()),
            None => Err("missing host".to_owned()),
        }
    }

    pub fn parse(url: &str) -> Result<Self, String> {
        // the url lib allows some weird stuff, so we fitler manually
        if url.contains("'") {
            return Err("Invalid character '".into());
        }

        let url_with_scheme = if !url.contains("://") {
            format!("http://{}", url)
        } else {
            url.into()
        };

        Url::parse(&url_with_scheme)
            .map_err(|err| format!("failed to parse url '{}': {}", url_with_scheme, err))
            .and_then(|url| Self::from_url(url))
    }

    fn unsafe_parse(url: &str) -> Self {
        Self::parse(url).unwrap()
    }
}

impl Display for SanitizedUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<SanitizedName> for SanitizedUrl {
    fn from(name: SanitizedName) -> Self {
        SanitizedUrl::unsafe_parse(&name.0)
    }
}

impl Into<SanitizedName> for SanitizedUrl {
    fn into(self) -> SanitizedName {
        SanitizedName(self.to_string())
    }
}

impl Channel for Rss {
    fn sanitize(&self, url: &str) -> Result<SanitizedName, String> {
        SanitizedUrl::parse(url).map(|u| u.into())
    }

    fn sanitize_for_db_search(&self, url: &str) -> Result<SanitizedName, String> {
        SanitizedUrl::parse(url).map(|u| SanitizedName(u.to_string_without_scheme()))
    }

    fn search(&self, query: SanitizedName) -> Result<Vec<ChannelInfo>, SearchError> {
        let url = SanitizedUrl::from(query).to_url();
        fetch_feeds(&url).map_err(|e| e.into())
    }

    fn fetch_updates(
        &self,
        _name: &str,
        url: &str,
        last_fetched: Option<DateTime<Utc>>,
    ) -> Result<Vec<Update>, String> {
        let rss_channel = RssChannel::from_url(url)
            .map_err(|err| format!("failed to fetch channel from url '{}': {:?}", url, err))?;
        let mut updates = Vec::with_capacity(rss_channel.items().len());
        for item in rss_channel.items() {
            let update = Update {
                title: item
                    .title()
                    .ok_or_else(|| format!("No title for {:?}", item))?
                    .to_owned(),
                url: item
                    .link()
                    .ok_or_else(|| format!("No url for {:?}", item))?
                    .to_owned(),
                // todo don't ignore parse error
                published: item
                    .pub_date()
                    .map(|date| {
                        DateTime::parse_from_rfc2822(date)
                            .map(|dt| dt.with_timezone(&Utc))
                            .map_err(|parse_err| {
                                format!("Failed to parse date '{}': {:?}", date, parse_err)
                            })
                    })
                    .ok_or_else(|| format!("No pub_date for {:?}", item))??,
            };
            // todo this is a technical error, which should be handled differently from the above business error
            let already_seen = last_fetched
                .map(|lf| lf > update.published)
                .unwrap_or(false);
            if already_seen {
                println!("Ignoring known update: {}", update.title);
            } else {
                updates.push(update)
            }
        }
        Ok(updates)
    }
}

#[derive(Debug)]
pub enum FeedError {
    NotFound(String),
    TechnicalError(String),
    UnknownError(String),
}

impl Into<SearchError> for FeedError {
    fn into(self) -> SearchError {
        use FeedError::*;
        match self {
            NotFound(_) => SearchError::ChannelNotFound,
            TechnicalError(msg) => SearchError::TechnicalError(msg),
            UnknownError(msg) => SearchError::TechnicalError(msg),
        }
    }
}

impl From<ToStrError> for FeedError {
    fn from(err: ToStrError) -> FeedError {
        FeedError::TechnicalError(format!(
            "header contains invisible ascii characters: {:?}",
            err
        ))
    }
}

impl From<RssError> for FeedError {
    fn from(err: RssError) -> FeedError {
        FeedError::TechnicalError(format!("Failed to parse XML as rss channel: {:?}", err))
    }
}

impl From<AtomError> for FeedError {
    fn from(err: AtomError) -> FeedError {
        FeedError::TechnicalError(format!("Failed to parse XML as atom feed: {:?}", err))
    }
}

impl From<reqwest::Error> for FeedError {
    fn from(err: reqwest::Error) -> FeedError {
        FeedError::TechnicalError(format!(
            "Failed to extract body from reqwest response: {:?}",
            err,
        ))
    }
}

pub fn fetch_feeds(full_url: &Url) -> Result<Vec<ChannelInfo>, FeedError> {
    use FeedError::*;

    let host = match full_url.host_str() {
        Some(host_str) => host_str,
        None => {
            return Err(TechnicalError(format!(
                "Missing host_str in url: {:?}",
                full_url
            )))
        }
    };

    let sane_url = format!("{}://{}{}", full_url.scheme(), host, full_url.path());

    let response = match reqwest::blocking::get(&sane_url) {
        Ok(resp) if resp.status() == StatusCode::OK => resp,
        Ok(resp) => return Err(NotFound(format!("Server returned code {}", resp.status()))),
        Err(err) => return Err(UnknownError(format!("Failed to fetch: {:?}", err))),
    };

    match response.headers().get(CONTENT_TYPE) {
        Some(c_type) if c_type.to_str()?.contains("text/html") => {
            let mut feeds = Vec::new();
            let body = response.text()?;
            let links = extract_feeds_from_html(&full_url, &body)?;
            println!("Links in HTML: {:?} --> recurse", links);
            for link in links {
                // fixme this could recurse forever. prevent that..
                let mut new_feeds = fetch_feeds(&link)?;
                feeds.append(&mut new_feeds);
            }
            Ok(feeds)
        }
        Some(c_type) if c_type.to_str()?.contains("application/xml") => {
            let buffer = BufReader::new(response);
            let feed: Feed = Feed::read_from(buffer)?;
            Ok(vec![ChannelInfo {
                name: feed.title().into(),
                url: sane_url.clone(),
                link: feed
                    .links()
                    .iter()
                    .next()
                    .map(|l| l.href().into())
                    .unwrap_or(sane_url),
            }])
        }
        Some(c_type) if c_type.to_str()?.contains("application/rss+xml") => {
            let buffer = BufReader::new(response);
            let channel: RssChannel = RssChannel::read_from(buffer)?;
            Ok(vec![ChannelInfo {
                name: channel.title().into(),
                url: sane_url,
                link: channel.link().into(),
            }])
        }
        Some(c_type) => Err(UnknownError(format!(
            "Unknown content type: {}",
            c_type.to_str()?
        ))),
        None => Err(UnknownError("No content type".into())),
    }
}

fn extract_feeds_from_html(url: &Url, html: &str) -> Result<Vec<Url>, FeedError> {
    let attribute_value = |atts: &kuchiki::Attributes, name: &str| -> Option<String> {
        atts.get(name).map(|v| v.to_owned())
    };
    let rss: String = "application/rss+xml".into();
    let mut links = Vec::new();
    let document = kuchiki::parse_html().one(html);
    let all_links = match document.select("link") {
        Err(err) => {
            return Err(FeedError::TechnicalError(format!(
                "failed to extract links from document: {:?}",
                err,
            )))
        }
        Ok(all_links) => all_links,
    };

    for link in all_links {
        let node: &kuchiki::NodeRef = link.as_node();
        if let Some(kuchiki::ElementData { attributes, .. }) = node.as_element() {
            if attribute_value(&attributes.borrow(), "type").contains(&rss) {
                if let Some(href) = attribute_value(&attributes.borrow(), "href") {
                    match url.join(&href) {
                        Ok(link_url) => links.push(link_url),
                        Err(err) => {
                            eprintln!("Failed to attach {} to base url {:?}: {:?}", href, url, err)
                        }
                    }
                } else {
                    eprintln!("link/rss tag without href. attributes: {:?}", attributes);
                }
            }
        }
    }

    Ok(links)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_atom_theverge_indirect() {
        let url = Url::parse("https://theverge.com").unwrap();
        let feeds = fetch_feeds(&url).expect("Failed to fetch feeds");
        assert_eq!(2, feeds.len());
        let all_posts = feeds
            .iter()
            .find(|f| f.name == "The Verge -  All Posts")
            .expect("Missing All Posts");
        assert_eq!(
            ChannelInfo {
                name: "The Verge -  All Posts".into(),
                url: "https://theverge.com/rss/index.xml".into(),
                link: "https://www.theverge.com/".into(),
            },
            *all_posts
        );
        let front_pages = feeds
            .iter()
            .find(|f| f.name == "The Verge -  Front Pages")
            .expect("Front Pages missing");
        assert_eq!(
            ChannelInfo {
                name: "The Verge -  Front Pages".into(),
                url: "https://www.theverge.com/rss/front-page/index.xml".into(),
                link: "https://www.theverge.com/".into(),
            },
            *front_pages
        );
    }

    #[test]
    fn fetch_atom_theverge_direct() {
        let url = Url::parse("https://theverge.com/rss/index.xml").unwrap();
        let feeds = fetch_feeds(&url).expect("Failed to fetch feeds");
        assert_eq!(1, feeds.len());
        let all_posts = feeds
            .iter()
            .find(|f| f.name == "The Verge -  All Posts")
            .expect("Missing All Posts");
        assert_eq!(
            ChannelInfo {
                name: "The Verge -  All Posts".into(),
                url: "https://theverge.com/rss/index.xml".into(),
                link: "https://www.theverge.com/".into(),
            },
            *all_posts
        );
    }

    #[test]
    fn fetch_rss_sedaily_direct() {
        let url =
            Url::parse("https://softwareengineeringdaily.com/category/podcast/feed/").unwrap();
        let feeds = fetch_feeds(&url).expect("Failed to fetch feeds");
        assert_eq!(1, feeds.len());
        let all_posts = feeds.iter().next().expect("Missing channel");
        assert_eq!(
            ChannelInfo {
                name: "Podcast – Software Engineering Daily".into(),
                url: "https://softwareengineeringdaily.com/category/podcast/feed/".into(),
                link: "https://softwareengineeringdaily.com".into(),
            },
            *all_posts
        );
    }

    #[test]
    fn fetch_rss_acolyer_indirect() {
        let url = Url::parse("https://blog.acolyer.org").unwrap();
        let feeds = fetch_feeds(&url).expect("Failed to fetch feeds");
        assert_eq!(2, feeds.len());
        println!("all: {:?}", feeds);
        let feed = feeds
            .iter()
            .find(|f| f.name == "the morning paper")
            .expect("Missing feed");
        assert_eq!(
            ChannelInfo {
                name: "the morning paper".into(),
                url: "https://blog.acolyer.org/feed/".into(),
                link: "https://blog.acolyer.org".into(),
            },
            *feed
        );
        let comments = feeds
            .iter()
            .find(|f| f.name == "Comments for the morning paper")
            .expect("Missing channel");
        assert_eq!(
            ChannelInfo {
                name: "Comments for the morning paper".into(),
                url: "https://blog.acolyer.org/comments/feed/".into(),
                link: "https://blog.acolyer.org".into(),
            },
            *comments
        );
    }

    #[test]
    fn extract_links_in_html() {
        let html = r"
          <!DOCTYPE html>
          <html lang='en'>
          <head>
          <link rel='alternate' type='application/rss+xml' title='Feed' href='https://blog.acolyer.org/feed/' />
          <link rel='alternate' type='application/rss+xml' title='Comments' href='/comments/feed/' />
          </head>
          <body>
          </body>
          </html>
        ";
        let base_url = Url::parse("https://blog.acolyer.org").unwrap();
        let feeds = extract_feeds_from_html(&base_url, html).expect("Failed to parse");
        assert_eq!(2, feeds.len());
        assert_eq!(
            Url::parse("https://blog.acolyer.org/feed/").unwrap(),
            feeds[0],
        );
        assert_eq!(
            Url::parse("https://blog.acolyer.org/comments/feed/").unwrap(),
            feeds[1],
        );
    }

    #[test]
    fn parse_validation_https() {
        assert_eq!(
            SanitizedUrl::parse("https://google.com/foo").map(|u| u.to_string()),
            Ok("https://google.com/foo".to_owned())
        )
    }

    #[test]
    fn parse_validation_http() {
        assert_eq!(
            SanitizedUrl::parse("http://google.com/foo").map(|u| u.to_string()),
            Ok("http://google.com/foo".to_owned())
        )
    }

    #[test]
    fn parse_validation_no_scheme() {
        assert_eq!(
            SanitizedUrl::parse("google.com").map(|u| u.to_string()),
            Ok("http://google.com/".to_owned())
        )
    }

    #[test]
    fn parse_validation_invalid_port() {
        assert_eq!(
            SanitizedUrl::parse("google.com:1234"),
            Err("cannot have port".to_owned())
        )
    }

    #[test]
    fn parse_validation_remove_query_string() {
        assert_eq!(
            SanitizedUrl::parse("http://google.com/foo?hello=world").map(|u| u.to_string()),
            Ok("http://google.com/foo".to_owned())
        )
    }

    #[test]
    fn parse_validation_remove_hash_with_path() {
        assert_eq!(
            SanitizedUrl::parse("http://google.com/foo#foo").map(|u| u.to_string()),
            Ok("http://google.com/foo".to_owned())
        )
    }

    #[test]
    fn parse_validation_remove_hash_without_path() {
        assert_eq!(
            SanitizedUrl::parse("http://google.com#foo").map(|u| u.to_string()),
            Ok("http://google.com/".to_owned())
        )
    }

    #[test]
    fn parse_validation_invalid_char() {
        assert_eq!(true, SanitizedUrl::parse("goo'gle.com").is_err())
    }

    #[test]
    fn parse_validation_reject_ip() {
        assert_eq!(
            SanitizedUrl::parse("http://127.0.0.1"),
            Err("cannot be ip".to_owned())
        )
    }

    #[test]
    fn parse_validation_reject_ftp() {
        assert_eq!(
            SanitizedUrl::parse("ftp://fms@example.com"),
            Err("invalid scheme: ftp".to_owned())
        )
    }

    #[test]
    fn parse_validation_reject_garbage() {
        assert_eq!(
            true,
            SanitizedUrl::parse("data:text/plain,Hello?World#").is_err()
        )
    }

    #[test]
    fn parse_validation_reject_garbage_asdf() {
        assert_eq!(SanitizedUrl::parse("asdf"), Err("missing tld".to_owned()))
    }

    #[test]
    fn parse_validation_reject_garbage_x_dot_x() {
        assert_eq!(SanitizedUrl::parse("x.x"), Err("missing tld".to_owned()))
    }

    #[test]
    fn sanitze_url_back_and_forth() {
        let original_string = "https://google.com/path/to/that";
        assert_eq!(
            original_string,
            SanitizedUrl::parse(original_string)
                .expect("failed to parse original")
                .to_string()
        )
    }
}
