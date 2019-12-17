use super::channel::*;

use chrono::{DateTime, Utc};
use rss::Channel as RssChannel;
use url::Url;

pub struct Rss {}

impl Channel for Rss {
    fn validate(&self, url: &str) -> Result<String, String> {
        sanitize_blog_url(url).map_err(|err| format!("url is invalid: {}", err))
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

#[cfg(test)]
mod tests {
    use super::*;

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
