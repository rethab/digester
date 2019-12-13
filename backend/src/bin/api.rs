#![feature(proc_macro_hygiene, decl_macro)]

extern crate backend;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

extern crate url;

use backend::db;
use backend::iam;

use rocket::fairing::AdHoc;
use rocket::http::Status as HttpStatus;
use rocket::http::{Cookie, Cookies, Method};
use rocket::request::{self, FromRequest, Request};
use rocket::response::status::Custom;
use rocket::response::{self, Responder};
use rocket::{Outcome, State};

use rocket_contrib::databases::diesel::PgConnection;
use rocket_contrib::databases::redis::Connection as RedisConnection;
use rocket_contrib::json::{Json, JsonValue};

use rocket_cors;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Error};

use time::Duration;
use url::Url;
use uuid::Uuid;

#[database("digester")]
struct DigesterDbConn(PgConnection);

#[database("redis")]
struct Redis(RedisConnection);

#[derive(Deserialize, Debug, PartialEq)]
struct NewBlog {
    url: String,
}

enum JsonResponse {
    Ok(JsonValue),
    BadRequest(String),
    InternalServerError,
    NotFound,
    Unauthorized,
}

struct Protected(iam::Session);

impl<'a, 'r> FromRequest<'a, 'r> for Protected {
    type Error = ();
    // todo check how failure can be handled
    fn from_request(req: &'a Request<'r>) -> request::Outcome<Protected, ()> {
        match Cookies::from_request(req) {
            Outcome::Success(cookies) => {
                match cookies.get("SESSION_ID") {
                    None => return Outcome::Forward(()),
                    Some(cookie) => {
                        // todo parse fails on garbage input, need to handle
                        let session_id =
                            Uuid::parse_str(cookie.value()).expect("failed to parse uuid");
                        let redis = Redis::from_request(req)?;
                        match iam::fetch_session(&redis, session_id) {
                            Ok(Some(session)) => Outcome::Success(Protected(session)),
                            Ok(None) => Outcome::Forward(()),
                            Err(_) => Outcome::Forward(()), // todo log
                        }
                    }
                }
            }
            _ => Outcome::Forward(()),
        }
    }
}

impl<'r> Responder<'r> for JsonResponse {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        let (body, status) = match self {
            JsonResponse::Ok(body) => (body, HttpStatus::Ok),
            JsonResponse::BadRequest(error) => (json!({ "error": error }), HttpStatus::BadRequest),
            JsonResponse::InternalServerError => (json!({}), HttpStatus::InternalServerError),
            JsonResponse::NotFound => (json!({}), HttpStatus::NotFound),
            JsonResponse::Unauthorized => (json!({}), HttpStatus::Unauthorized),
        };
        Custom(status, body).respond_to(req)
    }
}

#[derive(Deserialize, Debug, PartialEq)]
struct BlaBla {
    code: String,
    #[serde(rename = "clientId")]
    client_id: String,
    #[serde(rename = "redirectUri")]
    redirect_uri: String,
}

// creates the session cookie. a None value creates a removal cookie
fn create_session_cookie(maybe_id: Option<Uuid>) -> Cookie<'static> {
    let value = maybe_id
        .map(|id| {
            id.to_simple()
                .encode_lower(&mut Uuid::encode_buffer())
                .to_owned()
        })
        .unwrap_or(String::new());
    // todo review cookie settings
    Cookie::build("SESSION_ID", value)
        .domain("localhost")
        .secure(false)
        .path("/")
        .http_only(false)
        .max_age(Duration::days(1))
        .finish()
}

// todo how to prevent malicious users from calling this? (and us essentially being a github proxy)
#[post("/auth/github", data = "<oauth_data>")]
fn auth_github(
    db: DigesterDbConn,
    mut redis: Redis,
    mut cookies: Cookies,
    oauth_data: Json<BlaBla>,
    provider: State<iam::Github>,
) -> JsonResponse {
    use iam::AuthenticationError;
    let code = iam::AuthorizationCode(oauth_data.0.code);
    match iam::authenticate::<iam::Github>(&db.0, &mut redis.0, &provider, code) {
        Ok(session) => {
            let cookie = create_session_cookie(Some(session.id.clone()));
            cookies.add(cookie);
            JsonResponse::Ok(json!({
                "username": session.username,
            }))
        }
        Err(AuthenticationError::UnknownFailure(msg)) => {
            println!("Unknown auth failure: {}", msg);
            JsonResponse::InternalServerError
        }
        Err(AuthenticationError::TokenExchangeFailed(msg)) => {
            println!("token exchange failure: {}", msg);
            JsonResponse::InternalServerError
        }
    }
}

#[get("/auth/me")]
fn auth_me(session: Protected) -> JsonResponse {
    JsonResponse::Ok(json! ({
        "username": session.0.username
    }))
}

#[get("/auth/me", rank = 2)]
fn auth_unauthenticated() -> JsonResponse {
    JsonResponse::Unauthorized
}

#[post("/auth/logout")]
fn auth_logout(
    maybe_session: Option<Protected>,
    mut redis: Redis,
    mut cookies: Cookies,
) -> JsonResponse {
    match maybe_session {
        Some(session) => {
            println!("Destroying session");
            match iam::logout(&mut redis.0, session.0) {
                Ok(()) => (),
                Err(_) => return JsonResponse::InternalServerError,
            }
        }
        None => {
            println!("No session to destroy");
        }
    }
    let cookie = create_session_cookie(None);
    cookies.remove(cookie);
    JsonResponse::Ok(json!({}))
}

#[post("/blogs/add", data = "<new_blog>")]
fn add_blog(session: Protected, db: DigesterDbConn, new_blog: Json<NewBlog>) -> JsonResponse {
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

#[catch(500)]
fn internal_error() -> JsonResponse {
    JsonResponse::InternalServerError
}

#[catch(404)]
fn not_found() -> JsonResponse {
    JsonResponse::NotFound
}

fn main() -> Result<(), Error> {
    // todo properly implement CORS, this only works development
    let allowed_origins = AllowedOrigins::some_exact(&["http://localhost:8080"]);
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post, Method::Put]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()?;

    let config_reader = AdHoc::on_attach("Github Identity Provider", |rocket| {
        let github =
            iam::Github::from_rocket_config(rocket.config()).expect("Failed to read github config");
        Ok(rocket.manage(github))
    });

    rocket::ignite()
        .attach(DigesterDbConn::fairing())
        .attach(Redis::fairing())
        .attach(cors)
        .attach(config_reader)
        .register(catchers![internal_error, not_found])
        .mount(
            "/",
            routes![
                add_blog,
                auth_github,
                auth_me,
                auth_logout,
                auth_unauthenticated
            ],
        )
        .launch();

    Ok(())
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