#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

extern crate url;

use rocket::http::Status as HttpStatus;
use rocket::request::Request;
use rocket::response::status::Custom;
use rocket::response::{self, Responder};

use rocket_contrib::databases::postgres;
use rocket_contrib::json::{Json, JsonValue};

use url::Url;

#[database("digester")]
struct DigesterDbConn(postgres::Connection);

#[derive(Deserialize, Debug, PartialEq)]
struct NewBlog {
    url: String,
}

enum JsonResponse {
    Ok(JsonValue),
    BadRequest(String),
    InternalServerError,
    NotFound,
}

impl<'r> Responder<'r> for JsonResponse {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        let (body, status) = match self {
            JsonResponse::Ok(body) => (body, HttpStatus::Ok),
            JsonResponse::BadRequest(error) => (json!({ "error": error }), HttpStatus::BadRequest),
            JsonResponse::InternalServerError => (json!({}), HttpStatus::InternalServerError),
            JsonResponse::NotFound => (json!({}), HttpStatus::NotFound),
        };
        Custom(status, body).respond_to(req)
    }
}

#[post("/blogs/add", data = "<new_blog>")]
fn add_blog(conn: DigesterDbConn, new_blog: Json<NewBlog>) -> JsonResponse {
    match validate_blog(new_blog.0) {
        Ok(valid) => {
            match insert_blog(conn, valid) {
                // todo log
                Err(InsertError::Duplicate) => {
                    JsonResponse::BadRequest("blog already exists".to_owned())
                }
                Err(InsertError::Unknown) => JsonResponse::InternalServerError,
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

enum InsertError {
    Duplicate,
    Unknown,
}

fn insert_blog(conn: DigesterDbConn, valid_blog: ValidBlog) -> Result<u64, InsertError> {
    conn.execute("INSERT INTO blogs (url) VALUES ($1)", &[&valid_blog.0.url])
        .map_err(|err| {
            if err.code() == Some(&postgres::error::UNIQUE_VIOLATION) {
                InsertError::Duplicate
            } else {
                InsertError::Unknown
                // todo log
            }
        })
}

fn sanitize_blog_url(url: String) -> Result<String, String> {
    let url_with_scheme = if !url.contains("://") {
        format!("http://{}", url)
    } else {
        url
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
                Some(url::Host::Domain(d)) => Ok(d),
                Some(_ip) => Err("cannot be ip".to_owned()),
                None => Err("missing host".to_owned()),
            };

            maybe_scheme
                .and_then(|s| maybe_host.map(|h| (s, h)))
                .map(|(scheme, host)| format!("{}://{}{}", scheme, host, valid.path()))
        }
    }
}

#[catch(500)]
fn internal_error() -> JsonResponse {
    JsonResponse::InternalServerError
}

#[catch(404)]
fn not_found() -> JsonResponse {
    JsonResponse::NotFound
}

fn main() {
    rocket::ignite()
        .attach(DigesterDbConn::fairing())
        .register(catchers![internal_error, not_found])
        .mount("/", routes![add_blog])
        .launch();
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
}
