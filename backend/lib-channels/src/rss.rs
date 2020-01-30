use super::channel::*;

use atom_syndication::Error as AtomError;
use atom_syndication::Feed;
use chrono::naive::NaiveDateTime;
use chrono::{DateTime, Utc};
use kuchiki::traits::*;
use reqwest::blocking::{Client, Response};
use reqwest::header;
use reqwest::header::ToStrError;
use reqwest::StatusCode;
use rss::Channel as RssChannel;
use rss::Error as RssError;
use std::fmt::Display;
use std::io::{BufRead, BufReader};
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
    fn format_url(&self) -> String {
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
        if url.contains('\'') {
            return Err("Invalid character '".into());
        }

        let url_with_scheme = if !url.contains("://") {
            format!("http://{}", url)
        } else {
            url.into()
        };

        Url::parse(&url_with_scheme)
            .map_err(|err| format!("failed to parse url '{}': {}", url_with_scheme, err))
            .and_then(Self::from_url)
    }

    fn unsafe_parse(url: &str) -> Self {
        Self::parse(url).unwrap()
    }
}

impl Display for SanitizedUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.format_url())
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
        fetch_channel_info(&url, false).map_err(|e| e.into())
    }

    fn fetch_updates(
        &self,
        _name: &str,
        url: &str,
        last_fetched: Option<DateTime<Utc>>,
    ) -> Result<Vec<Update>, String> {
        let resp = fetch_resource(url)
            .map_err(|err| format!("Failed to fetch url '{}': {:?}", url, err))?;

        let updates = match parse_feed(resp) {
            Ok(ParsedFeed::Rss(rss)) => rss_to_updates(&rss)?,
            Ok(ParsedFeed::Atom(atom)) => atom_to_updates(&atom)?,
            Err(err) => return Err(format!("Failed to parse '{}': {:?}", url, err)),
        };

        Ok(updates
            .into_iter()
            .filter(|u| !u.is_old(last_fetched))
            .collect())
    }
}

fn rss_to_updates(channel: &RssChannel) -> Result<Vec<Update>, String> {
    let mut updates = Vec::with_capacity(channel.items().len());
    for item in channel.items() {
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
                .ok_or(format!("Missing pub_date for {:?}", item))
                .and_then(parse_pub_date)?,
        };

        updates.push(update);
    }
    Ok(updates)
}

fn atom_to_updates(feed: &Feed) -> Result<Vec<Update>, String> {
    let mut updates = Vec::with_capacity(feed.entries().len());
    for entry in feed.entries() {
        let update = Update {
            title: entry.title().into(),
            url: atom_link(entry.links()).unwrap_or_else(|| format!("No links for {:?}", entry)),
            published: entry
                .published()
                .cloned()
                .unwrap_or(entry.updated().clone()) // eg. wikipedia doesn't use published
                .with_timezone(&Utc),
        };
        updates.push(update);
    }
    Ok(updates)
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
            NotFound(msg) => SearchError::ChannelNotFound(msg),
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

fn fetch_channel_info(full_url: &Url, recursed: bool) -> Result<Vec<ChannelInfo>, FeedError> {
    let sane_url = full_url.to_string();
    let response = fetch_resource(&sane_url)?;

    if is_html(&response) {
        // if we didn't prevent this, we might recurse forever if a page
        // points to itself (maliciously or not..)
        if recursed {
            return Err(FeedError::TechnicalError(format!(
                "Url {} points to html, but we already recursed",
                sane_url,
            )));
        }
        let mut feeds = Vec::new();
        let body = response.text()?;
        let links = extract_feeds_from_html(&full_url, &body)?;
        println!("Links in HTML: {:?} --> recurse", links);
        for link in links {
            let new_feeds = fetch_channel_info(&link, true)?;
            for new_feed in new_feeds {
                if is_new_feed(&feeds, &new_feed) {
                    feeds.push(new_feed);
                } else {
                    println!("Ignoring duplicate feed: {:?}", new_feed);
                }
            }
        }
        Ok(feeds)
    } else {
        match parse_feed(response) {
            Ok(ParsedFeed::Rss(channel)) => Ok(vec![ChannelInfo {
                name: channel.title().into(),
                url: sane_url,
                link: channel.link().into(),
            }]),
            Ok(ParsedFeed::Atom(feed)) => Ok(vec![ChannelInfo {
                name: feed.title().into(),
                url: sane_url.clone(),
                link: atom_link(feed.links()).unwrap_or(sane_url),
            }]),
            Err(err) => Err(FeedError::UnknownError(format!(
                "Neither atom nor rss: {:?}",
                err
            ))),
        }
    }
}

// returns false if we already have the same feed. specifically,
// we're trying to filter equivalent feeds that are exposed as
// rss and atom
fn is_new_feed(feeds: &[ChannelInfo], new_feed: &ChannelInfo) -> bool {
    for feed in feeds {
        if feed.url == new_feed.url {
            // exactly the same, not even different type
            return false;
        } else if feed.name == new_feed.name && feed.link == new_feed.link {
            // same title and pointing to same website --> most likely same
            return false;
        }
    }
    return true;
}

fn fetch_resource(url: &str) -> Result<Response, FeedError> {
    use FeedError::*;

    let mut builder = Client::builder().gzip(true).build()?.get(url);
    builder = builder.header(
        header::USER_AGENT,
        "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:72.0) Gecko/20100101 Firefox/72.0",
    );
    builder = builder.header(header::ACCEPT_ENCODING, "gzip");

    match builder.send() {
        Ok(resp) if resp.status() == StatusCode::OK => Ok(resp),
        Ok(resp) => Err(NotFound(format!("Server returned code {}", resp.status()))),
        Err(err) => Err(UnknownError(format!("Failed to fetch: {:?}", err))),
    }
}

fn extract_feeds_from_html(url: &Url, html: &str) -> Result<Vec<Url>, FeedError> {
    let attribute_value = |atts: &kuchiki::Attributes, name: &str| -> Option<String> {
        atts.get(name).map(|v| v.to_owned())
    };

    let is_feed_link = |maybe_type: Option<String>| -> bool {
        maybe_type
            .map(|t| match t.as_str() {
                "application/rss+xml" => true,
                "application/atom+xml" => true,
                _ => false,
            })
            .unwrap_or(false)
    };

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
            let link_type = attribute_value(&attributes.borrow(), "type");
            if is_feed_link(link_type) {
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

fn is_html(resp: &Response) -> bool {
    c_type(resp).contains("text/html")
}

enum ParsedFeed {
    Atom(Box<Feed>),
    Rss(Box<RssChannel>),
}

impl ParsedFeed {
    fn parse_rss<R: BufRead>(buffer: R) -> Result<ParsedFeed, String> {
        RssChannel::read_from(buffer)
            .map(|rss| ParsedFeed::Rss(Box::new(rss)))
            .map_err(|err| format!("Failed to parse as rss: {:?}", err))
    }

    fn parse_atom<R: BufRead>(buffer: R) -> Result<ParsedFeed, String> {
        Feed::read_from(buffer)
            .map(|feed| ParsedFeed::Atom(Box::new(feed)))
            .map_err(|err| format!("Failed to parse as atom: {:?}", err))
    }
}

fn parse_feed(mut resp: Response) -> Result<ParsedFeed, String> {
    if is_rss(&resp) {
        let buffer = BufReader::new(resp);
        ParsedFeed::parse_rss(buffer)
    } else if is_atom(&resp) {
        let buffer = BufReader::new(resp);
        ParsedFeed::parse_atom(buffer)
    } else if is_xml(&resp) {
        // if we don't know the content type, we take a look at the beginning
        // of the body and look for xml tags for rss or atom.
        // this is a bit cumbersome/inefficient, because `reqwest.Response`
        // doesn't allow us to seek back to the beginning after having looked
        // at the first few bytes in the response. therefore, we copy the entire
        // response into a byte vector, which then allows us to peek at the
        // first few bytes.
        let mut bytes = Vec::with_capacity(resp.content_length().unwrap_or(512) as usize);
        resp.copy_to(&mut bytes)
            .map_err(|err| format!("Failed to copy buffer: {:?}", err))?;
        let contents = peek_buffer(&bytes);
        let buffer = BufReader::with_capacity(bytes.len(), &bytes[..]);
        if contents.contains("<rss") {
            ParsedFeed::parse_rss(buffer)
        } else if contents.contains("<feed") {
            ParsedFeed::parse_atom(buffer)
        } else {
            Err(format!(
                "XML response doesn't contain <rss or <feed in the first few bytes: {:?}",
                bytes
            ))
        }
    } else {
        Err(format!("Unhandled content type: {}", c_type(&resp)))
    }
}

fn peek_buffer(bytes: &[u8]) -> String {
    String::from_utf8_lossy(&bytes[0..256]).into()
}

fn is_rss(resp: &Response) -> bool {
    c_type(resp).contains("application/rss+xml")
}

fn is_atom(resp: &Response) -> bool {
    c_type(resp).contains("application/atom+xml")
}

fn is_xml(resp: &Response) -> bool {
    c_type(resp).contains("application/xml") || c_type(resp).contains("text/xml")
}

fn c_type(resp: &Response) -> String {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned()
}

fn atom_link(links: &[atom_syndication::Link]) -> Option<String> {
    let mut found_link: Option<String> = None;
    for link in links {
        // for youtube, this contains the link to the channel while anoter link
        // with rel=self points to the feed
        if link.rel() == "alternate" || found_link.is_none() {
            found_link = Some(link.href().into());
        }
    }
    found_link
}

fn parse_pub_date(datetime: &str) -> Result<DateTime<Utc>, String> {
    if datetime.contains('+') {
        DateTime::parse_from_rfc2822(datetime)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|parse_err| {
                format!(
                    "Failed to parse date '{}' as rfc2822: {:?}",
                    datetime, parse_err
                )
            })
    } else {
        NaiveDateTime::parse_from_str(datetime, "%a, %d %b %Y %H:%M:%S")
            .map(|naive| DateTime::from_utc(naive, Utc))
            .map_err(|parse_err| {
                format!(
                    "Failed to parse date '{}' with custom format: {:?}",
                    datetime, parse_err
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn fetch_atom_theverge_indirect() {
        let url = Url::parse("https://theverge.com").unwrap();
        let feeds = fetch_channel_info(&url, false).expect("Failed to fetch feeds");
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
    fn fetch_refuse_to_recurse() {
        let url = Url::parse("https://theverge.com").unwrap();
        assert_eq!(true, fetch_channel_info(&url, true).is_err())
    }

    #[test]
    fn fetch_atom_theverge_direct() {
        let url = Url::parse("https://theverge.com/rss/index.xml").unwrap();
        let feeds = fetch_channel_info(&url, false).expect("Failed to fetch feeds");
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
        let feeds = fetch_channel_info(&url, false).expect("Failed to fetch feeds");
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
        let feeds = fetch_channel_info(&url, false).expect("Failed to fetch feeds");
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
    fn fetch_rss_nytimes_indirect() {
        let url = Url::parse("https://nytimes.com").unwrap();
        let feeds = fetch_channel_info(&url, false).expect("Failed to fetch feeds");
        assert_eq!(1, feeds.len());
        let feed = feeds[0].clone();
        assert_eq!(
            ChannelInfo {
                name: "NYT > Top Stories".into(),
                url: "https://rss.nytimes.com/services/xml/rss/nyt/HomePage.xml".into(),
                link: "https://www.nytimes.com?emc=rss&partner=rss".into(),
            },
            feed
        );
    }

    #[test]
    fn fetch_atom_youtube_direct() {
        let url = Url::parse(
            "https://www.youtube.com/feeds/videos.xml?channel_id=UCxec_VgCE-5DUZ8MocKbEdg",
        )
        .unwrap();
        let feeds = fetch_channel_info(&url, false).expect("Failed to fetch feeds");
        assert_eq!(1, feeds.len());
        let feed = feeds[0].clone();
        assert_eq!(
            ChannelInfo {
                name: "marktcheck".into(),
                url: "https://www.youtube.com/feeds/videos.xml?channel_id=UCxec_VgCE-5DUZ8MocKbEdg"
                    .into(),
                link: "https://www.youtube.com/channel/UCxec_VgCE-5DUZ8MocKbEdg".into(),
            },
            feed
        );
    }

    #[test]
    fn fetch_atom_rss_200ok_deduplicate() {
        let url = Url::parse("https://200ok.ch/").unwrap();
        let feeds = fetch_channel_info(&url, false).unwrap();
        assert_eq!(1, feeds.len());
        let feed = feeds[0].clone();
        assert_eq!(
            ChannelInfo {
                name: "200ok - Consultancy, Research Lab, Incubator".into(),
                url: "https://200ok.ch/rss.xml".into(),
                link: "https://200ok.ch/".into(),
            },
            feed
        );
    }

    #[test]
    fn fetch_rss_wpbeginner_bot_protection_without_user_agent() {
        let url = Url::parse("https://www.wpbeginner.com/blog/").unwrap();
        let feeds = fetch_channel_info(&url, false).unwrap();
        assert_eq!(2, feeds.len());
    }

    #[test]
    fn extract_links_in_html() {
        let html = r"
          <!DOCTYPE html>
          <html lang='en'>
          <head>
          <link rel='alternate' type='application/rss+xml' title='Feed' href='https://blog.acolyer.org/feed/' />
          <link rel='alternate' type='application/rss+xml' title='Comments' href='/comments/feed/' />
          <link rel='alternate' type='application/atom+xml' title='AppSignal Blog atom feed' href='https://blog.appsignal.com/feed.xml' >
          </head>
          <body>
          </body>
          </html>
        ";
        let base_url = Url::parse("https://blog.acolyer.org").unwrap();
        let feeds = extract_feeds_from_html(&base_url, html).expect("Failed to parse");
        assert_eq!(3, feeds.len());
        assert_eq!(
            Url::parse("https://blog.acolyer.org/feed/").unwrap(),
            feeds[0],
        );
        assert_eq!(
            Url::parse("https://blog.acolyer.org/comments/feed/").unwrap(),
            feeds[1],
        );
        assert_eq!(
            Url::parse("https://blog.appsignal.com/feed.xml").unwrap(),
            feeds[2],
        );
    }

    #[test]
    fn parse_datetime_without_timezone() {
        // example: https://craftcms.com/blog.rss
        let actual = parse_pub_date("Tue, 10 Dec 2019 16:00:00").expect("Failed to parse date");
        let expected = Utc.ymd(2019, 12, 10).and_hms(16, 0, 0);
        assert_eq!(expected, actual)
    }

    #[test]
    fn parse_datetime_with_timezone_utc() {
        // example: https://softwareengineeringdaily.com/category/podcast/feed/
        let actual =
            parse_pub_date("Tue, 28 Jan 2020 10:00:48 +0000").expect("Failed to parse date");
        let expected = Utc.ymd(2020, 1, 28).and_hms(10, 0, 48);
        assert_eq!(expected, actual)
    }

    #[test]
    fn parse_datetime_with_timezone_offset() {
        let actual =
            parse_pub_date("Tue, 28 Jan 2020 10:00:48 +0200").expect("Failed to parse date");
        let expected = Utc.ymd(2020, 1, 28).and_hms(8, 0, 48);
        assert_eq!(expected, actual)
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
