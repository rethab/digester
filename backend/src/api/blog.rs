extern crate url;

use super::super::db;

use super::common::*;

use rocket::Rocket;

use rocket_contrib::json::Json;
use serde::Deserialize;

use url::Url;

pub fn mount(rocket: Rocket) -> Rocket {
    rocket.mount("/blogs", routes![add])
}

#[derive(Deserialize, Debug, PartialEq)]
struct NewBlog {
    url: String,
}

#[post("/add", data = "<new_blog>")]
fn add(session: Protected, db: DigesterDbConn, new_blog: Json<NewBlog>) -> JsonResponse {
    println!("User {} is making a request :)", session.0.username);
    match validate_blog(new_blog.0) {
        Ok(valid) => {
            match insert_blog(db, valid) {
                // todo log
                Err(db::InsertError::Duplicate) => {
                    JsonResponse::BadRequest("blog already exists".to_owned())
                }
                Err(db::InsertError::Unknown) => JsonResponse::InternalServerError,
                Ok(_id) => JsonResponse::Ok(json!({ "added": true })),
            }
        }
        Err(err_msg) => JsonResponse::BadRequest(err_msg),
    }
}

#[derive(Debug, PartialEq)]
struct ValidBlog(NewBlog);

fn validate_blog(mut new_blog: NewBlog) -> Result<ValidBlog, String> {
    match sanitize_blog_url(new_blog.url) {
        Err(err) => Err(format!("url is invalid: {}", err)),
        Ok(valid_url) => {
            new_blog.url = valid_url;
            Ok(ValidBlog(new_blog))
        }
    }
}

fn insert_blog(db: DigesterDbConn, valid_blog: ValidBlog) -> Result<(), db::InsertError> {
    db::blogs_insert(
        &db.0,
        db::NewBlog {
            url: valid_blog.0.url,
        },
    )
}

fn sanitize_blog_url(url: String) -> Result<String, String> {
    let url_with_scheme = if !url.contains("://") {
        format!("http://{}", url)
    } else {
        url
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
            validate_blog(NewBlog {
                url: "https://google.com/foo".to_owned(),
            }),
            Ok(ValidBlog(NewBlog {
                url: "https://google.com/foo".to_owned(),
            }))
        )
    }

    #[test]
    fn blog_validation_http() {
        assert_eq!(
            validate_blog(NewBlog {
                url: "http://google.com/foo".to_owned(),
            }),
            Ok(ValidBlog(NewBlog {
                url: "http://google.com/foo".to_owned(),
            }))
        )
    }

    #[test]
    fn blog_validation_no_scheme() {
        assert_eq!(
            validate_blog(NewBlog {
                url: "google.com".to_owned(),
            }),
            Ok(ValidBlog(NewBlog {
                url: "http://google.com/".to_owned(),
            }))
        )
    }

    #[test]
    fn blog_validation_invalid_port() {
        assert_eq!(
            validate_blog(NewBlog {
                url: "google.com:1234".to_owned(),
            }),
            Err("url is invalid: cannot have port".to_owned())
        )
    }

    #[test]
    fn blog_validation_remove_query_string() {
        assert_eq!(
            validate_blog(NewBlog {
                url: "http://google.com/foo?hello=world".to_owned(),
            }),
            Ok(ValidBlog(NewBlog {
                url: "http://google.com/foo".to_owned(),
            }))
        )
    }

    #[test]
    fn blog_validation_remove_hash_with_path() {
        assert_eq!(
            validate_blog(NewBlog {
                url: "http://google.com/foo#foo".to_owned(),
            }),
            Ok(ValidBlog(NewBlog {
                url: "http://google.com/foo".to_owned(),
            }))
        )
    }

    #[test]
    fn blog_validation_remove_hash_without_path() {
        assert_eq!(
            validate_blog(NewBlog {
                url: "http://google.com#foo".to_owned(),
            }),
            Ok(ValidBlog(NewBlog {
                url: "http://google.com/".to_owned(),
            }))
        )
    }

    #[test]
    fn blog_validation_reject_ip() {
        assert_eq!(
            validate_blog(NewBlog {
                url: "http://127.0.0.1".to_owned(),
            }),
            Err("url is invalid: cannot be ip".to_owned())
        )
    }

    #[test]
    fn blog_validation_reject_ftp() {
        assert_eq!(
            validate_blog(NewBlog {
                url: "ftp://fms@example.com".to_owned(),
            }),
            Err("url is invalid: invalid scheme: ftp".to_owned())
        )
    }

    #[test]
    fn blog_validation_reject_garbage() {
        assert_eq!(
            validate_blog(NewBlog {
                url: "data:text/plain,Hello?World#".to_owned(),
            }),
            Err("url is invalid: not a url".to_owned())
        )
    }

    #[test]
    fn blog_validation_reject_garbage_asdf() {
        assert_eq!(
            validate_blog(NewBlog {
                url: "asdf".to_owned(),
            }),
            Err("url is invalid: missing tld".to_owned())
        )
    }

    #[test]
    fn blog_validation_reject_garbage_x_dot_x() {
        assert_eq!(
            validate_blog(NewBlog {
                url: "x.x".to_owned(),
            }),
            Err("url is invalid: missing tld".to_owned())
        )
    }
}
