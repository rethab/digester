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

use rocket::http::Status as HttpStatus;
use rocket::http::{Cookie, Cookies, Method, SameSite};
use rocket::request::Request;
use rocket::response::status::Custom;
use rocket::response::{self, Redirect, Responder};

use rocket_contrib::databases::diesel::PgConnection;
use rocket_contrib::json::{Json, JsonValue};

use rocket_oauth2::hyper_sync_rustls_adapter::HyperSyncRustlsAdapter;
use rocket_oauth2::{OAuth2, TokenResponse};

use rocket_cors;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Error};

use hyper::{
    header::{qitem, Accept, Authorization, UserAgent},
    mime::Mime,
    net::HttpsConnector,
    Client,
};

use std::io::Read;
use url::Url;

#[database("digester")]
struct DigesterDbConn(PgConnection);

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

#[derive(serde::Deserialize)]
struct GitHubUserInfo {
    #[serde(default)]
    name: String,
}

fn github_callback(
    request: &Request,
    token: TokenResponse,
) -> Result<Redirect, Box<dyn (::std::error::Error)>> {
    let https = HttpsConnector::new(hyper_sync_rustls::TlsClient::new());
    let client = Client::with_connector(https);

    // Use the token to retrieve the user's GitHub account information.
    let mime: Mime = "application/vnd.github.v3+json"
        .parse()
        .expect("parse GitHub MIME type");
    let response = client
        .get("https://api.github.com/user")
        .header(Authorization(format!("token {}", token.access_token())))
        .header(Accept(vec![qitem(mime)]))
        .header(UserAgent("rocket_oauth2 demo application".into()))
        .send()?;

    if !response.status.is_success() {
        return Err(format!("got non-success status {}", response.status).into());
    }

    let user_info: GitHubUserInfo = serde_json::from_reader(response.take(2 * 1024 * 1024))?;

    // todo remove expect
    let mut cookies = request.guard::<Cookies>().expect("request cookies");

    cookies.add_private(
        Cookie::build("username", user_info.name)
            .same_site(SameSite::Lax)
            .finish(),
    );
    Ok(Redirect::to("/"))
}

#[derive(Deserialize, Debug, PartialEq)]
struct BlaBla {
    code: String,
    clientId: String,
    redirectUri: String,
}

#[post("/auth/github", data = "<oauth_data>")]
fn auth_github(oauth_data: Json<BlaBla>) -> Result<JsonResponse, Box<dyn (::std::error::Error)>> {
    println!("data from github: {:?}", oauth_data);
    // github_callback(request.0, token_response.into_inner())

    Ok(JsonResponse::Ok(json!({
        "redirect_to": "http://localhost:8080/auth/callback",
        "access_token": "my-access-token"
    })))
}

#[post("/blogs/add", data = "<new_blog>")]
fn add_blog(db: DigesterDbConn, new_blog: Json<NewBlog>) -> JsonResponse {
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

    let oauth = OAuth2::fairing(
        HyperSyncRustlsAdapter,
        github_callback,
        "github",
        "/auth/github",
        Some(("/login/github", vec!["user:read".to_string()])),
    );

    rocket::ignite()
        .attach(DigesterDbConn::fairing())
        .attach(cors)
        .attach(oauth)
        .register(catchers![internal_error, not_found])
        .mount("/", routes![add_blog, auth_github])
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
