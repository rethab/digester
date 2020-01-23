use super::channel::*;

use chrono::{DateTime, Utc};
use reqwest::header::{ToStrError, CONTENT_TYPE};
use reqwest::StatusCode;
use rss::Channel as RssChannel;
use rss::Error as RssError;
use std::io::BufReader;
use url::Url;
use xml::attribute::OwnedAttribute;
use xml::reader::{EventReader, XmlEvent};

pub struct Rss {}

impl Channel for Rss {
    fn validate(&self, url: &str) -> Result<String, ValidationError> {
        sanitize_blog_url(url)
            .map_err(|err| ValidationError::ChannelInvalid(format!("url is invalid: {}", err)))
    }

    fn fetch_updates(
        &self,
        url: &str,
        last_fetched: Option<DateTime<Utc>>,
    ) -> Result<Vec<Update>, String> {
        let rss_channel = RssChannel::from_url(url)
            .map_err(|err| format!("failed to fetch channel from url '{}': {:?}", url, err))?;
        println!(
            "Found {} articles for channel {}",
            rss_channel.items().len(),
            url
        );
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

fn sanitize_blog_url(url: &str) -> Result<String, String> {
    let url_with_scheme = if !url.contains("://") {
        format!("http://{}", url)
    } else {
        url.into()
    };

    let minimum_length = |s: &str| {
        let pieces: Vec<&str> = s.split('.').collect();
        pieces.len() >= 2 && pieces.last().unwrap().len() >= 2
    };

    match Url::parse(&url_with_scheme) {
        Err(err) => {
            eprintln!("failed to parse url '{}': {}", url_with_scheme, err);
            Err("not a url".to_owned())
        }
        Ok(valid) if valid.port().is_some() => Err("cannot have port".to_owned()),
        Ok(valid) => {
            let maybe_scheme = match valid.scheme() {
                "http" | "https" => Ok(valid.scheme()),
                scheme => Err(format!("invalid scheme: {}", scheme)),
            };
            let maybe_host = match valid.host() {
                Some(url::Host::Domain(d)) if minimum_length(d) => Ok(d),
                Some(url::Host::Domain(_)) => Err("missing tld".to_owned()),
                Some(_ip) => Err("cannot be ip".to_owned()),
                None => Err("missing host".to_owned()),
            };

            maybe_scheme
                .and_then(|s| maybe_host.map(|h| (s, h)))
                .map(|(scheme, host)| format!("{}://{}{}", scheme, host, valid.path()))
        }
    }
}

#[derive(Debug)]
enum FeedError {
    NotFound(String),
    TechnicalError(String),
    UnknownError(String),
}

#[derive(PartialEq, Debug)]
struct FeedInfo {
    title: String, // title of the feed
    url: String,   // url of this feed (eg. theverge.com/feed.xml)
    link: String,  // website of this feed (eg. theverge.com)
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

impl From<reqwest::Error> for FeedError {
    fn from(err: reqwest::Error) -> FeedError {
        FeedError::TechnicalError(format!(
            "Failed to extract body from reqwest response: {:?}",
            err,
        ))
    }
}

fn fetch_feeds(full_url: &Url) -> Result<Vec<FeedInfo>, FeedError> {
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
            Err(TechnicalError("atom feeds are not supported yet".into()))
        }
        Some(c_type) if c_type.to_str()?.contains("application/rss+xml") => {
            let buffer = BufReader::new(response);
            let channel: RssChannel = RssChannel::read_from(buffer)?;
            Ok(vec![FeedInfo {
                title: channel.title().into(),
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
    let attribute_value = |atts: &[OwnedAttribute], name: &str| -> Option<String> {
        atts.iter()
            .find(|att| att.name.local_name == name)
            .map(|att| att.value.clone())
    };
    let rss: String = "application/rss+xml".into();
    let mut links = Vec::new();
    let parser = EventReader::from_str(html);
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) if name.local_name == "link" => {
                if attribute_value(&attributes, "type").contains(&rss) {
                    if let Some(href) = attribute_value(&attributes, "href") {
                        match url.join(&href) {
                            Ok(link_url) => links.push(link_url),
                            Err(err) => eprintln!(
                                "Failed to attach {} to base url {:?}: {:?}",
                                href, url, err
                            ),
                        }
                    } else {
                        eprintln!("link/rss tag without href. attributes: {:?}", attributes);
                    }
                }
            }
            Ok(XmlEvent::EndElement { name }) if name.local_name == "head" => break,
            Err(e) => {
                eprintln!("Parser failed on {}: {:?}", html, e);
                break;
            }
            _ => {}
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
            .find(|f| f.title == "The Verge - All Posts")
            .expect("Missing All Posts");
        assert_eq!(
            FeedInfo {
                title: "The Verge - All Posts".into(),
                url: "https://theverge.com/rss/index.xml".into(),
                link: "https://theverge.com".into(),
            },
            *all_posts
        );
        let front_pages = feeds
            .iter()
            .find(|f| f.title == "The Verge - Front Pages")
            .expect("Front Pages missing");
        assert_eq!(
            FeedInfo {
                title: "The Verge - Front Page".into(),
                url: "https://theverge.com/rss/front-page/index.xml".into(),
                link: "https://theverge.com".into(),
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
            .find(|f| f.title == "The Verge - All Posts")
            .expect("Missing All Posts");
        assert_eq!(
            FeedInfo {
                title: "The Verge - All Posts".into(),
                url: "https://theverge.com/rss/index.xml".into(),
                link: "https://theverge.com".into(),
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
            FeedInfo {
                title: "Podcast – Software Engineering Daily".into(),
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
        assert_eq!(1, feeds.len());
        let feed = feeds
            .iter()
            .find(|f| f.title == "the morning paper » Feed")
            .expect("Missing feed");
        assert_eq!(
            FeedInfo {
                title: "the morning paper".into(),
                url: "https://blob.acolyer.org/feed".into(),
                link: "https://blog.acolyer.org".into(),
            },
            *feed
        );
        let comments = feeds
            .iter()
            .find(|f| f.title == "the morning paper » Comments Feed")
            .expect("Missing channel");
        assert_eq!(
            FeedInfo {
                title: "Comments for the morning paper".into(),
                url: "https://blob.acolyer.org/comments/feed".into(),
                link: "https://blog.acolyer.org".into(),
            },
            *comments
        );
    }

    #[test]
    fn blog_validation_https() {
        assert_eq!(
            sanitize_blog_url("https://google.com/foo"),
            Ok("https://google.com/foo".to_owned())
        )
    }

    #[test]
    fn blog_validation_http() {
        assert_eq!(
            sanitize_blog_url("http://google.com/foo"),
            Ok("http://google.com/foo".to_owned())
        )
    }

    #[test]
    fn blog_validation_no_scheme() {
        assert_eq!(
            sanitize_blog_url("google.com"),
            Ok("http://google.com/".to_owned())
        )
    }

    #[test]
    fn blog_validation_invalid_port() {
        assert_eq!(
            sanitize_blog_url("google.com:1234"),
            Err("cannot have port".to_owned())
        )
    }

    #[test]
    fn blog_validation_remove_query_string() {
        assert_eq!(
            sanitize_blog_url("http://google.com/foo?hello=world"),
            Ok("http://google.com/foo".to_owned())
        )
    }

    #[test]
    fn blog_validation_remove_hash_with_path() {
        assert_eq!(
            sanitize_blog_url("http://google.com/foo#foo"),
            Ok("http://google.com/foo".to_owned())
        )
    }

    #[test]
    fn blog_validation_remove_hash_without_path() {
        assert_eq!(
            sanitize_blog_url("http://google.com#foo"),
            Ok("http://google.com/".to_owned())
        )
    }

    #[test]
    fn blog_validation_reject_ip() {
        assert_eq!(
            sanitize_blog_url("http://127.0.0.1"),
            Err("cannot be ip".to_owned())
        )
    }

    #[test]
    fn blog_validation_reject_ftp() {
        assert_eq!(
            sanitize_blog_url("ftp://fms@example.com"),
            Err("invalid scheme: ftp".to_owned())
        )
    }

    #[test]
    fn blog_validation_reject_garbage() {
        assert_eq!(
            sanitize_blog_url("data:text/plain,Hello?World#"),
            Err("not a url".to_owned())
        )
    }

    #[test]
    fn blog_validation_reject_garbage_asdf() {
        assert_eq!(sanitize_blog_url("asdf"), Err("missing tld".to_owned()))
    }

    #[test]
    fn blog_validation_reject_garbage_x_dot_x() {
        assert_eq!(sanitize_blog_url("x.x"), Err("missing tld".to_owned()))
    }
}
