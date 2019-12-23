use super::super::iam;

use rocket::http::Status as HttpStatus;
use rocket::http::{Cookies, Method};
use rocket::request::{self, FromRequest, Request};
use rocket::response::status::Custom;
use rocket::response::{self, Responder};
use rocket::{self, Config, Outcome};

use rocket_contrib::databases::diesel::PgConnection;
use rocket_contrib::databases::redis::Connection as RedisConnection;
use rocket_contrib::json::JsonValue;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};
use uuid::Uuid;

#[database("digester")]
pub struct DigesterDbConn(pub PgConnection);

#[database("redis")]
pub struct Redis(pub RedisConnection);

pub enum JsonResponse {
    Ok(JsonValue),
    BadRequest(String),
    InternalServerError,
    NotFound,
    Unauthorized,
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

pub struct Protected(pub iam::Session);

impl<'a, 'r> FromRequest<'a, 'r> for Protected {
    type Error = ();
    // todo check how failure can be handled
    fn from_request(req: &'a Request<'r>) -> request::Outcome<Protected, ()> {
        match Cookies::from_request(req) {
            Outcome::Success(cookies) => {
                match cookies.get("SESSION_ID") {
                    None => Outcome::Forward(()),
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

pub fn cors_fairing(config: &Config) -> Result<Cors, String> {
    // todo properly implement CORS, this only works development
    let allowed_origin: &str = config
        .get_table("cors")
        .expect("Missing config entry cors")
        .get("allowed_origin")
        .expect("Missing config entry 'cors.allowed_origin'")
        .as_str()
        .expect("Missing config entry cors.allowed_origin");
    let allowed_origins = AllowedOrigins::some_exact(&[allowed_origin]);
    CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post, Method::Put]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .map_err(|err| format!("Failed to setup CORS: {:?}", err))
}
