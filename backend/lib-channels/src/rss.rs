use super::channel::*;

use atom_syndication::Error as AtomError;
use atom_syndication::Feed;
use chrono::naive::{NaiveDate, NaiveDateTime};
use chrono::{DateTime, Utc};
use core::time::Duration;
use kuchiki::iter::{Descendants, Elements, Select};
use kuchiki::traits::*;
use reqwest::blocking::{Client, Response};
use reqwest::header;
use reqwest::header::ToStrError;
use reqwest::StatusCode;
use rss::Error as RssError;
use rss::{Channel as RssChannel, Item as RssItem};
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
        fetch_channel_info(&url, 0).map_err(|e| e.into())
    }

    fn fetch_updates(&self, url: &str) -> Result<Vec<Update>, String> {
        let resp = fetch_resource(url)
            .map_err(|err| format!("Failed to fetch url '{}': {:?}", url, err))?;

        parse_feed(resp)
            .map_err(|err| format!("Failed to parse '{}': {:?}", url, err))
            .and_then(|feed| match feed {
                ParsedFeed::Rss(rss) => rss_to_updates(&rss),
                ParsedFeed::Atom(atom) => atom_to_updates(&atom),
            })
    }
}

fn rss_to_updates(channel: &RssChannel) -> Result<Vec<Update>, String> {
    let mut updates = Vec::with_capacity(channel.items().len());
    for item in channel.items() {
        let maybe_date = item.pub_date().or_else(|| rss_dc_date(&item));
        match maybe_date {
            None => println!(
                "Neither pub_date nor dc:date for item {} in channel {} ({})",
                item.title().unwrap_or("???"),
                channel.title(),
                channel.link()
            ),
            Some(date) => {
                let update = Update {
                    ext_id: None,
                    title: item
                        .title()
                        .ok_or_else(|| format!("No title for {:?}", item))?
                        .to_owned(),
                    url: item
                        .link()
                        .map(|l| make_absolute(channel.link(), l))
                        .ok_or_else(|| format!("No url for {:?}", item))?
                        .to_owned(),
                    published: parse_pub_date(date)?,
                };
                updates.push(update);
            }
        }
    }
    Ok(updates)
}

fn atom_to_updates(feed: &Feed) -> Result<Vec<Update>, String> {
    let mut updates = Vec::with_capacity(feed.entries().len());
    for entry in feed.entries() {
        let update = Update {
            ext_id: None,
            title: entry.title().into(),
            url: atom_article_link(feed.links(), entry.links())
                .unwrap_or_else(|| format!("No links for {:?}", entry)),
            published: entry
                .published()
                .cloned()
                .unwrap_or_else(|| *entry.updated()) // eg. wikipedia doesn't use published
                .with_timezone(&Utc),
        };
        updates.push(update);
    }
    Ok(updates)
}

fn make_absolute(feed_link: &str, url: &str) -> String {
    if url.starts_with('/') {
        format!("{}{}", feed_link, url)
    } else {
        url.into()
    }
}

#[derive(Debug)]
pub enum FeedError {
    NotFound(String),
    TechnicalError(String),
    UnknownError(String),
    Timeout(String),
}

impl Into<SearchError> for FeedError {
    fn into(self) -> SearchError {
        use FeedError::*;
        match self {
            NotFound(msg) => SearchError::ChannelNotFound(msg),
            Timeout(msg) => SearchError::Timeout(msg),
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

// recursed determines how many times we already recursively
// called this method. we want to prevent infinite recursion
// because an html page could point to itself and then we'd
// never get out of here
fn fetch_channel_info(full_url: &Url, recursed: u8) -> Result<Vec<ChannelInfo>, FeedError> {
    let sane_url = full_url.to_string();
    let response = fetch_resource(&sane_url)?;

    if is_html(&response) {
        // if we didn't prevent this, we might recurse forever if a page
        // points to itself (maliciously or not..)
        if recursed > 2 {
            println!(
                "Url {} points to html, but we already recursed {}",
                sane_url, recursed
            );
            return Ok(Vec::new());
        }
        let mut feeds = Vec::new();
        let body = response.text()?;
        let links = extract_feeds_from_html(&full_url, &body)?;
        println!("Links in HTML: {:?} --> recurse({})", links, recursed);
        for link in links {
            let channel_infos = fetch_channel_info(&link, recursed + 1)?;
            for channel_info in channel_infos {
                add_channel_info(&mut feeds, channel_info)
            }
        }
        Ok(feeds)
    } else {
        match parse_feed(response) {
            Ok(ParsedFeed::Rss(channel)) => Ok(vec![ChannelInfo {
                name: channel.title().into(),
                ext_id: sane_url,
                link: channel.link().into(),
                verified: false,
            }]),
            Ok(ParsedFeed::Atom(feed)) => Ok(vec![ChannelInfo {
                name: feed.title().into(),
                ext_id: sane_url.clone(),
                link: atom_link(feed.links()).unwrap_or(sane_url),
                verified: false,
            }]),
            Err(err) => Err(FeedError::UnknownError(format!(
                "Neither atom nor rss: {:?}",
                err
            ))),
        }
    }
}

// adds the new feed to the existing feed if it is new or replaces
// an existing if it is the same feed in a newer format. For example,
// if we have an rss feed for site x and `new_feed` is an atom feed
// for the same page, the rss feed will be replaced
fn add_channel_info(feeds: &mut Vec<ChannelInfo>, new_feed: ChannelInfo) {
    let mut maybe_duplicate: Option<usize> = None;
    for (index, feed) in feeds.iter().enumerate() {
        if
        // exactly the same, not even different type
        feed.ext_id == new_feed.ext_id ||
        // same title and pointing to same website --> most likely same
        (feed.name == new_feed.name && feed.link == new_feed.link)
        {
            maybe_duplicate = Some(index);
            break;
        }
    }

    if let Some(index) = maybe_duplicate {
        if is_better(&new_feed, &feeds[index]) {
            feeds[index] = new_feed;
        } else {
            println!("Ignoring duplicate feed: {:?}", new_feed);
        }
    } else {
        feeds.push(new_feed);
    }
}

// returns true if the new feed is better and should replace the old one
fn is_better(new_feed: &ChannelInfo, existing: &ChannelInfo) -> bool {
    let guess_atom = |feed: &ChannelInfo| {
        feed.name.to_ascii_lowercase().contains("atom")
            || feed.ext_id.to_ascii_lowercase().contains("atom")
    };

    guess_atom(new_feed) && !guess_atom(existing)
}

fn fetch_resource(url: &str) -> Result<Response, FeedError> {
    use FeedError::*;

    let timeout = Duration::from_secs(3);
    let mut builder = Client::builder()
        .gzip(true)
        .timeout(timeout)
        .build()?
        .get(url);
    builder = builder.header(
        header::USER_AGENT,
        "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:72.0) Gecko/20100101 Firefox/72.0",
    );
    builder = builder.header(header::ACCEPT_ENCODING, "gzip");

    match builder.send() {
        Ok(resp) if resp.status() == StatusCode::OK => Ok(resp),
        Ok(resp) => Err(NotFound(format!(
            "Server returned code {} for url {}",
            resp.status(),
            url
        ))),
        Err(err) if format!("{:?}", err).contains("Name or service not known") => {
            // todo I guess the above could be improved ;-)
            Err(NotFound(format!("DNS lookup failed: {:?}", err)))
        }
        Err(err) if err.is_timeout() => Err(Timeout(format!(
            "Failed to fetch resource within {:?}: {:?}",
            timeout, err
        ))),
        Err(err) => Err(UnknownError(format!("Failed to fetch: {:?}", err))),
    }
}

fn extract_feeds_from_html(url: &Url, html: &str) -> Result<Vec<Url>, FeedError> {
    // create absolute urls from relative urls
    let mk_urls = |links: Vec<String>| {
        let mut urls = Vec::with_capacity(links.len());
        for link in links {
            match url.join(&link) {
                Ok(link_url) => urls.push(link_url),
                Err(err) => eprintln!("Ignoring invalid link '{}': {:?}", link, err),
            }
        }
        urls
    };

    let document = kuchiki::parse_html().one(html);

    // meta rss/atom links
    match document.select("link") {
        Err(err) => {
            return Err(FeedError::TechnicalError(format!(
                "failed to extract links from document: {:?}",
                err,
            )))
        }
        Ok(link_tags) => {
            let head_links = extract_feeds_from_html_link(link_tags);
            if !head_links.is_empty() {
                return Ok(mk_urls(head_links));
            }
        }
    };

    match document.select("div[data-rss-url]") {
        Err(err) => {
            return Err(FeedError::TechnicalError(format!(
                "failed to extract div[data-rss-url] from document: {:?}",
                err,
            )))
        }
        Ok(div_tags) => {
            let div_links = extract_feeds_from_html_attribute(div_tags, "data-rss-url");
            if !div_links.is_empty() {
                return Ok(mk_urls(div_links));
            }
        }
    };

    match document.select("meta[property='article:author']") {
        Err(err) => {
            return Err(FeedError::TechnicalError(format!(
                "failed to extract meta[property='article:author'] from document: {:?}",
                err,
            )))
        }
        Ok(meta_tags) => {
            let meta_tags = extract_feeds_from_html_attribute(meta_tags, "content");
            if !meta_tags.is_empty() {
                return Ok(mk_urls(meta_tags));
            }
        }
    };

    // fuzzy search in html body links (eg. <a href="myfeed.xml">RSS</a>)
    match document.select("a") {
        Err(err) => {
            return Err(FeedError::TechnicalError(format!(
                "failed to extract ankers from document: {:?}",
                err,
            )))
        }
        Ok(a_tags) => {
            let links = extract_potential_feed_link(a_tags);
            if !links.is_empty() {
                return Ok(mk_urls(links));
            }
        }
    }

    Ok(Vec::new())
}

fn extract_feeds_from_html_attribute(
    div_tags: Select<Elements<Descendants>>,
    attr_name: &str,
) -> Vec<String> {
    let mut links: Vec<String> = Vec::new();
    for div in div_tags {
        let div: &kuchiki::NodeRef = div.as_node();
        if let Some(kuchiki::ElementData { attributes, .. }) = div.as_element() {
            let attr_value = attributes.borrow().get(attr_name).map(|v| v.to_owned());
            match attr_value {
                Some(link_value) => links.push(link_value),
                None => eprintln!("Missing value in '{}': {:?}", attr_name, div),
            }
        }
    }

    links
}

fn extract_feeds_from_html_link(link_tags: Select<Elements<Descendants>>) -> Vec<String> {
    let is_feed_link = |maybe_type: Option<String>| -> bool {
        maybe_type
            .map(|t| match t.as_str() {
                "application/rss+xml" => true,
                "application/atom+xml" => true,
                _ => false,
            })
            .unwrap_or(false)
    };

    let mut links: Vec<String> = Vec::new();
    for link in link_tags {
        let node: &kuchiki::NodeRef = link.as_node();
        if let Some(kuchiki::ElementData { attributes, .. }) = node.as_element() {
            let link_type = attributes.borrow().get("type").map(|v| v.to_owned());
            if is_feed_link(link_type) {
                if let Some(href) = attributes.borrow().get("href") {
                    links.push(href.into());
                } else {
                    eprintln!("link/rss tag without href. attributes: {:?}", attributes);
                }
            }
        }
    }

    links
}

fn extract_potential_feed_link(anker_tags: Select<Elements<Descendants>>) -> Vec<String> {
    let fuzzy_feed_match =
        |ct: &str| -> bool { ct.contains("rss") || ct.contains("feed") || ct.contains("atom") };

    let mut links: Vec<String> = Vec::new();
    for anker in anker_tags {
        let node: &kuchiki::NodeRef = anker.as_node();
        if let Some(kuchiki::ElementData { attributes, .. }) = node.as_element() {
            let a1 = attributes.borrow();
            let mut attrs = a1
                .get("href")
                .into_iter()
                .chain(a1.get("title").into_iter())
                .chain(a1.get("class").into_iter());
            if attrs.any(fuzzy_feed_match) {
                if let Some(href) = a1.get("href") {
                    links.push(href.into());
                }
            }
        }
    }

    links
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
        if bytes.starts_with(&[31, 139, 8]) {
            // happens if a website returns a gzip'd response w/o
            // setting the header. eg. https://onlineitguru.com/blog/feed
            Err(format!(
                "Looks like a gzip response in disguise: {:?}",
                bytes
            ))
        } else if contents.contains("<rss") || contents.contains("<rdf:RDF") {
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
    c_type(resp).contains("application/")
        && (c_type(resp).contains("rss") || c_type(resp).contains("rdf"))
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

fn atom_article_link(
    feed_links: &[atom_syndication::Link],
    item_links: &[atom_syndication::Link],
) -> Option<String> {
    let feed_link = atom_link(feed_links);
    let item_link = atom_link(item_links);

    match item_link {
        None => None,
        Some(link) if link.starts_with('/') => feed_link.map(|fl| make_absolute(&fl, &link)),
        Some(link) => Some(link),
    }
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

fn rss_dc_date(item: &RssItem) -> Option<&str> {
    if let Some(ext) = item.dublin_core_ext() {
        match ext.dates().iter().next() {
            Some(date) => Some(date),
            None => None,
        }
    } else {
        None
    }
}

fn parse_pub_date(datetime: &str) -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc2822(datetime)
        .or_else(|_| DateTime::parse_from_rfc3339(datetime))
        .or_else(|_| DateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S%.3f %z"))
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            NaiveDateTime::parse_from_str(datetime, "%a, %d %b %Y %H:%M:%S")
                .or_else(|_| {
                    NaiveDate::parse_from_str(datetime, "%Y-%m-%d").map(|d| d.and_hms(0, 0, 0))
                })
                .map(|naive| DateTime::from_utc(naive, Utc))
        })
        .map_err(|parse_err| format!("Failed to parse date '{}' {:?}", datetime, parse_err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use reqwest::blocking::get;

    #[test]
    fn fetch_atom_theverge_indirect() {
        let url = Url::parse("https://theverge.com").unwrap();
        let feeds = fetch_channel_info(&url, 0).expect("Failed to fetch feeds");
        assert_eq!(2, feeds.len());
        let all_posts = feeds
            .iter()
            .find(|f| f.name == "The Verge -  All Posts")
            .expect("Missing All Posts");
        assert_eq!(
            ChannelInfo {
                name: "The Verge -  All Posts".into(),
                ext_id: "https://theverge.com/rss/index.xml".into(),
                link: "https://www.theverge.com/".into(),
                verified: false,
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
                ext_id: "https://www.theverge.com/rss/front-page/index.xml".into(),
                link: "https://www.theverge.com/".into(),
                verified: false,
            },
            *front_pages
        );
    }

    #[test]
    fn fetch_refuse_to_recurse() {
        let url = Url::parse("https://theverge.com").unwrap();
        let feeds = fetch_channel_info(&url, 3).expect("Failed to fetch feeds");
        assert_eq!(0, feeds.len())
    }

    #[test]
    fn fetch_atom_theverge_direct() {
        let url = Url::parse("https://theverge.com/rss/index.xml").unwrap();
        let feeds = fetch_channel_info(&url, 0).expect("Failed to fetch feeds");
        assert_eq!(1, feeds.len());
        let all_posts = feeds
            .iter()
            .find(|f| f.name == "The Verge -  All Posts")
            .expect("Missing All Posts");
        assert_eq!(
            ChannelInfo {
                name: "The Verge -  All Posts".into(),
                ext_id: "https://theverge.com/rss/index.xml".into(),
                link: "https://www.theverge.com/".into(),
                verified: false,
            },
            *all_posts
        );
    }

    #[test]
    fn fetch_atom_laravel_direct() {
        let url = Url::parse("https://blog.laravel.com/feed").unwrap();
        let feeds = fetch_channel_info(&url, 0).expect("Failed to fetch feeds");
        assert!(feeds.len() > 0);
    }

    #[test]
    fn fetch_rss_sedaily_direct() {
        let url =
            Url::parse("https://softwareengineeringdaily.com/category/podcast/feed/").unwrap();
        let feeds = fetch_channel_info(&url, 0).expect("Failed to fetch feeds");
        assert_eq!(1, feeds.len());
        let all_posts = feeds.iter().next().expect("Missing channel");
        assert_eq!(
            ChannelInfo {
                name: "Podcast – Software Engineering Daily".into(),
                ext_id: "https://softwareengineeringdaily.com/category/podcast/feed/".into(),
                link: "https://softwareengineeringdaily.com".into(),
                verified: false,
            },
            *all_posts
        );
    }

    #[test]
    fn fetch_rss_acolyer_indirect() {
        let url = Url::parse("https://blog.acolyer.org").unwrap();
        let feeds = fetch_channel_info(&url, 0).expect("Failed to fetch feeds");
        assert_eq!(2, feeds.len());
        println!("all: {:?}", feeds);
        let feed = feeds
            .iter()
            .find(|f| f.name == "the morning paper")
            .expect("Missing feed");
        assert_eq!(
            ChannelInfo {
                name: "the morning paper".into(),
                ext_id: "https://blog.acolyer.org/feed/".into(),
                link: "https://blog.acolyer.org".into(),
                verified: false,
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
                ext_id: "https://blog.acolyer.org/comments/feed/".into(),
                link: "https://blog.acolyer.org".into(),
                verified: false,
            },
            *comments
        );
    }

    #[test]
    fn fetch_rss_nytimes_indirect() {
        let url = Url::parse("https://nytimes.com").unwrap();
        let feeds = fetch_channel_info(&url, 0).expect("Failed to fetch feeds");
        assert_eq!(1, feeds.len());
        let feed = feeds[0].clone();
        assert_eq!(
            ChannelInfo {
                name: "NYT > Top Stories".into(),
                ext_id: "https://rss.nytimes.com/services/xml/rss/nyt/HomePage.xml".into(),
                link: "https://www.nytimes.com".into(),
                verified: false,
            },
            feed
        );
    }
    #[test]
    fn fetch_rss_medium_via_article_indirect() {
        // article points to author page, points to rss
        let url = Url::parse(
            "https://medium.com/@nikitonsky/medium-is-a-poor-choice-for-blogging-bb0048d19133",
        )
        .unwrap();
        let feeds = fetch_channel_info(&url, 0).expect("Failed to fetch feeds");
        assert_eq!(1, feeds.len());
        let feed = feeds[0].clone();
        assert_eq!(
            ChannelInfo {
                name: "Stories by Nikitonsky on Medium".into(),
                ext_id: "https://medium.com/feed/@nikitonsky".into(),
                link: "https://medium.com/@nikitonsky?source=rss-5247cb846abe------2".into(),
                verified: false,
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
        let feeds = fetch_channel_info(&url, 0).expect("Failed to fetch feeds");
        assert_eq!(1, feeds.len());
        let feed = feeds[0].clone();
        assert_eq!(
            ChannelInfo {
                name: "marktcheck".into(),
                ext_id:
                    "https://www.youtube.com/feeds/videos.xml?channel_id=UCxec_VgCE-5DUZ8MocKbEdg"
                        .into(),
                link: "https://www.youtube.com/channel/UCxec_VgCE-5DUZ8MocKbEdg".into(),
                verified: false,
            },
            feed
        );
    }

    #[test]
    fn fetch_atom_rss_200ok_deduplicate() {
        let url = Url::parse("https://200ok.ch/").unwrap();
        let feeds = fetch_channel_info(&url, 0).unwrap();
        assert_eq!(1, feeds.len());
        let feed = feeds[0].clone();
        assert_eq!(
            ChannelInfo {
                name: "200ok - Consultancy, Research Lab, Incubator".into(),
                ext_id: "https://200ok.ch/atom.xml".into(),
                link: "https://200ok.ch/".into(),
                verified: false,
            },
            feed
        );
    }

    #[test]
    fn fetch_rss_wpbeginner_bot_protection_without_user_agent() {
        let url = Url::parse("https://www.wpbeginner.com/blog/").unwrap();
        let feeds = fetch_channel_info(&url, 0).unwrap();
        assert_eq!(2, feeds.len());
    }

    #[test]
    fn fetch_rss_nos_even_when_recursion_limit_is_reached() {
        // points to feeds html page, which points feeds as well as itself. this
        // is to ensure that even if we reach the recursion limit, the feeds
        // that have been found are returned.
        let url = Url::parse("https://nos.nl").unwrap();
        let feeds = fetch_channel_info(&url, 0).expect("Failed to fetch feeds");
        assert!(feeds.len() > 5);
    }

    #[test]
    fn fetch_rss_artima_xrss_direct() {
        let url = Url::parse("https://www.artima.com/weblogs/feeds/bloggers/guido.rss").unwrap();
        let feeds = fetch_channel_info(&url, 0).unwrap();
        assert_eq!(1, feeds.len());
    }

    #[test]
    fn fetch_rdf_content_type_xml_slashdot_indirect() {
        let url = Url::parse("https://slashdot.org/").unwrap();
        let feeds = fetch_channel_info(&url, 0).unwrap();
        let feed = feeds[0].clone();
        assert_eq!(
            ChannelInfo {
                name: "Slashdot".into(),
                ext_id: "http://rss.slashdot.org/Slashdot/slashdotMain".into(),
                link: "https://slashdot.org/".into(),
                verified: false,
            },
            feed
        );
    }

    #[test]
    fn fetch_rdf_content_type_rdf_hasbrouck_direct() {
        let url = Url::parse("http://hasbrouck.org/blog/index.rdf").unwrap();
        let feeds = fetch_channel_info(&url, 0).unwrap();
        assert_eq!(1, feeds.len())
    }

    #[test]
    fn fetch_partial_updates_with_dates() {
        // the rss feed contains meta articles which are missing the pubDate field
        let url = "https://webkid.io/rss.xml";
        let rss = Rss {};
        let updates = rss.fetch_updates(url).expect("Failed to fetch");
        assert_eq!(false, updates.is_empty())
    }

    #[test]
    fn fetch_rdf_updates() {
        // this blog uses 'Dublin Core Metadata Initiative' (dc:date)
        let url = "https://hasbrouck.org/blog/index.rdf";
        let rss = Rss {};
        let updates = rss.fetch_updates(url).expect("Failed to fetch");
        assert_eq!(false, updates.is_empty())
    }

    #[test]
    fn fetch_updates_with_relative_links() {
        let url = "https://blog.appsignal.com/feed.xml";
        let rss = Rss {};
        let updates = rss.fetch_updates(url).expect("Failed to fetch");
        let post_url = &updates[0].url;
        assert_eq!(200, get(post_url).expect("Failed to fetch").status())
    }

    #[test]
    fn fetch_updates_with_absolute_links() {
        let url = "https://200ok.ch/rss.xml";
        let rss = Rss {};
        let updates = rss.fetch_updates(url).expect("Failed to fetch");
        let post_url = &updates[0].url;
        assert_eq!(200, get(post_url).expect("Failed to fetch").status())
    }

    #[test]
    fn extract_links_in_html_head() {
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
    fn extract_links_in_html_body_div() {
        let html = r"
          <!DOCTYPE html><html lang='en'><head></head><body>
            <div data-rss-url='https://www.toptal.com/blog.rss' data-title='Toptal Blog: Business, Design, and Technology' data-url='https://www.toptal.com/blog'>
          </body></html>
        ";
        let base_url = Url::parse("https://toptal.com").unwrap();
        let feeds = extract_feeds_from_html(&base_url, html).expect("Failed to parse");
        assert_eq!(1, feeds.len());
        assert_eq!(
            Url::parse("https://www.toptal.com/blog.rss").unwrap(),
            feeds[0],
        );
    }

    #[test]
    fn extract_links_in_html_body_a() {
        // eg. codepen.io
        let html = r"
          <!DOCTYPE html><html lang='en'><head></head><body>
            <div data-rss-url='https://www.toptal.com/blog.rss' data-title='Toptal Blog: Business, Design, and Technology' data-url='https://www.toptal.com/blog'>
          </body></html>
        ";
        let base_url = Url::parse("https://toptal.com").unwrap();
        let feeds = extract_feeds_from_html(&base_url, html).expect("Failed to parse");
        assert_eq!(1, feeds.len());
        assert_eq!(
            Url::parse("https://www.toptal.com/blog.rss").unwrap(),
            feeds[0],
        );
    }

    #[test]
    fn extract_links_in_html_head_meta() {
        // medium uses this thing on articles, which then points to the author's page,
        // which contains an rss link
        let html = r"
          <!DOCTYPE html><html lang='en'><head>
            <meta data-rh='true' property='article:author' content='https://medium.com/@nikitonsky'/>
          </head><body></body></html>
        ";
        let base_url = Url::parse(
            "https://medium.com/@nikitonsky/medium-is-a-poor-choice-for-blogging-bb0048d19133",
        )
        .unwrap();
        let feeds = extract_feeds_from_html(&base_url, html).expect("Failed to parse");
        assert_eq!(1, feeds.len());
        assert_eq!(
            Url::parse("https://medium.com/@nikitonsky").unwrap(),
            feeds[0],
        );
    }

    #[test]
    fn extract_links_from_a_hrefs() {
        let html = r"
          <!DOCTYPE html><html lang='en'><head></head><body>
          <app-root ng-version='5.2.2'><site-gtm><div class='wrapper'><use></use><a _ngcontent-c17='' href='https://cloudblog.withgoogle.com/rss/'><span _ngcontent-c17=''>RSS Feed</span><svg _ngcontent-c17='' aria-hidden='true' ><use _ngcontent-c17='' xlink:href='#mi-rss-feed' xmlns:xlink='http://www.w3.org/1999/xlink'></use></svg></a></div></site-gtm></app-root>
          <h1>TiDB and TiKV Blog<a class='rss' href='/blog/index.xml' title='RSS Feed'><svg xmlns='http://www.w3.org/2000/svg' width='13' height='17' viewBox=''><g fill=''><path d=''/><path d=''/></g></svg></a></h1>
          <div class='col-sm-6'> <h3><a class='float-right' href='https://www.keycloak.org/rss.xml'><img src='resources/images/rss.png'/></a></h3> </div>
          </body></html>
        ";
        let base_url = Url::parse("https://pingcap.com/blog/").unwrap();
        let feeds = extract_feeds_from_html(&base_url, html).expect("Failed to parse");
        assert_eq!(3, feeds.len());
        assert_eq!(
            Url::parse("https://cloudblog.withgoogle.com/rss/").unwrap(),
            feeds[0],
        );
        assert_eq!(
            Url::parse("https://pingcap.com/blog/index.xml").unwrap(),
            feeds[1],
        );
        assert_eq!(
            Url::parse("https://www.keycloak.org/rss.xml").unwrap(),
            feeds[2],
        );
    }

    #[test]
    fn prefer_links_from_meta_over_ahrefs() {
        let html = r"
          <!DOCTYPE html><html lang='en'>
          <head><link rel='alternate' type='application/rss+xml' title='Feed' href='https://blog.acolyer.org/feed/' /></head>
          <body><h1>TiDB and TiKV Blog<a class='rss' href='/blog/index.xml' title='RSS Feed'><svg xmlns='http://www.w3.org/2000/svg' width='13' height='17' viewBox=''><g fill=''><path d=''/><path d=''/></g></svg></a></h1></body>
          </html>
        ";
        let base_url = Url::parse("https://blog.acolyer.org").unwrap();
        let feeds = extract_feeds_from_html(&base_url, html).expect("Failed to parse");
        assert_eq!(1, feeds.len());
        assert_eq!(
            Url::parse("https://blog.acolyer.org/feed/").unwrap(),
            feeds[0],
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
    fn parse_datetime_with_timezone_gmt() {
        // example: the guardian
        let actual = parse_pub_date("Thu, 30 Jan 2020 09:29:07 GMT").expect("Failed to parse date");
        let expected = Utc.ymd(2020, 1, 30).and_hms(9, 29, 07);
        assert_eq!(expected, actual)
    }

    #[test]
    fn parse_datetime_with_timezone_est() {
        // example: https://blog.burntsushi.net/index.xml
        let actual = parse_pub_date("Mon, 27 Jan 2020 17:55:00 EST").expect("Failed to parse date");
        let expected = Utc.ymd(2020, 1, 27).and_hms(22, 55, 0);
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
    fn parse_datetime_with_timezone_negative_offset() {
        let actual =
            parse_pub_date("Tue, 28 Jan 2020 10:00:48 -0200").expect("Failed to parse date");
        let expected = Utc.ymd(2020, 1, 28).and_hms(12, 0, 48);
        assert_eq!(expected, actual)
    }

    #[test]
    fn parse_datetime_rfc_8601() {
        let actual = parse_pub_date("2020-01-31T10:51:50+02:00").expect("Failed to parse date");
        let expected = Utc.ymd(2020, 1, 31).and_hms(8, 51, 50);
        assert_eq!(expected, actual);

        let actual = parse_pub_date("2020-01-31T10:51:50Z").expect("Failed to parse date");
        let expected = Utc.ymd(2020, 1, 31).and_hms(10, 51, 50);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_datetime_with_spaces() {
        let actual = parse_pub_date("2020-05-12 11:09:25 +0800").expect("Failed to parse date");
        let expected = Utc.ymd(2020, 5, 12).and_hms(3, 9, 25);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_datetime_date_only() {
        let actual = parse_pub_date("2020-01-31").expect("Failed to parse date");
        let expected = Utc.ymd(2020, 1, 31).and_hms(0, 0, 0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn compare_channel_infos() {
        let rss_hint_in_url = ChannelInfo {
            name: "blog a".into(),
            ext_id: "https://a.ch/rss.xml".into(),
            link: "https://a.ch/".into(),
            verified: false,
        };
        let atom_hint_in_url = ChannelInfo {
            name: "blog a".into(),
            ext_id: "https://a.ch/atom.xml".into(),
            link: "https://a.ch/".into(),
            verified: false,
        };
        let rss_hint_in_name = ChannelInfo {
            name: "blog a rss feed".into(),
            ext_id: "https://a.ch/feed.xml".into(),
            link: "https://a.ch/".into(),
            verified: false,
        };
        let atom_hint_in_name = ChannelInfo {
            name: "blog a atom feed".into(),
            ext_id: "https://a.ch/feed.xml".into(),
            link: "https://a.ch/".into(),
            verified: false,
        };
        let other_blog_with_no_hint = ChannelInfo {
            name: "blog b".into(),
            ext_id: "https://b.ch/feed.xml".into(),
            link: "https://b.ch/".into(),
            verified: false,
        };
        let other_blog_with_rss_hint = ChannelInfo {
            name: "blog c rss feed".into(),
            ext_id: "https://c.ch/feed.xml".into(),
            link: "https://c.ch/".into(),
            verified: false,
        };

        // SCENARIO 1: Atom replaces Rss (hint in URL)
        let mut feeds: Vec<ChannelInfo> = Vec::new();
        add_channel_info(&mut feeds, rss_hint_in_url.clone());
        assert_eq!(1, feeds.len());

        // not add duplicate
        add_channel_info(&mut feeds, rss_hint_in_url.clone());
        assert_eq!(1, feeds.len());

        // replace with atom
        add_channel_info(&mut feeds, atom_hint_in_url.clone());
        assert_eq!(1, feeds.len());
        assert!(feeds.contains(&atom_hint_in_url));

        // not replace with rss
        add_channel_info(&mut feeds, rss_hint_in_url.clone());
        assert_eq!(1, feeds.len());
        assert!(feeds.contains(&atom_hint_in_url));

        // SCENARIO 2: Atom replaces Rss (hint in name)
        let mut feeds: Vec<ChannelInfo> = Vec::new();
        add_channel_info(&mut feeds, rss_hint_in_name.clone());
        add_channel_info(&mut feeds, atom_hint_in_name.clone());
        assert_eq!(1, feeds.len());
        assert!(feeds.contains(&atom_hint_in_name));

        // SCENARIO 3: add unrelated blog
        let mut feeds: Vec<ChannelInfo> = Vec::new();
        add_channel_info(&mut feeds, atom_hint_in_name.clone());
        add_channel_info(&mut feeds, other_blog_with_rss_hint.clone());
        add_channel_info(&mut feeds, other_blog_with_no_hint.clone());
        assert_eq!(3, feeds.len());

        // SCENARIO 4: distinct rss feeds for same website
        let mut feeds: Vec<ChannelInfo> = Vec::new();
        add_channel_info(&mut feeds, rss_hint_in_name.clone());
        add_channel_info(&mut feeds, rss_hint_in_url.clone());
        assert_eq!(2, feeds.len());
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
