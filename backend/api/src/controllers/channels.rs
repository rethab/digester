use lib_channels as channels;
use lib_db as db;

use super::common::*;
use channels::github_release::GithubRelease;
use channels::twitter::Twitter;
use channels::*;
use either::{Left, Right};
use rocket::http::RawStr;
use rocket::request::{FromFormValue, FromParam};
use rocket::Rocket;
use rocket::State;
use rocket_contrib::json::JsonValue;

use std::convert::{From, TryFrom, TryInto};

pub fn mount(rocket: Rocket) -> Rocket {
    rocket.mount("/channels", routes![show, search])
}

pub struct GithubApiToken(pub String);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
enum ChannelType {
    RssFeed,
    GithubRelease,
    Twitter,
    List,
}

impl<'v> FromFormValue<'v> for ChannelType {
    type Error = String;
    fn from_form_value(form_value: &'v RawStr) -> Result<ChannelType, String> {
        form_value.as_str().try_into()
    }
}

impl<'v> FromParam<'v> for ChannelType {
    type Error = String;
    fn from_param(param: &'v RawStr) -> Result<ChannelType, String> {
        param.as_str().try_into()
    }
}

impl From<db::ChannelType> for ChannelType {
    fn from(ct: db::ChannelType) -> ChannelType {
        match ct {
            db::ChannelType::GithubRelease => ChannelType::GithubRelease,
            db::ChannelType::RssFeed => ChannelType::RssFeed,
            db::ChannelType::Twitter => ChannelType::Twitter,
        }
    }
}

impl TryFrom<&str> for ChannelType {
    type Error = String;

    fn try_from(ct: &str) -> Result<ChannelType, String> {
        match ct {
            "GithubRelease" => Ok(ChannelType::GithubRelease),
            "RssFeed" => Ok(ChannelType::RssFeed),
            "Twitter" => Ok(ChannelType::Twitter),
            "List" => Ok(ChannelType::List),
            other => Err(format!("Invalid channel type: {}", other)),
        }
    }
}

impl From<channels::ChannelType> for ChannelType {
    fn from(ct: channels::ChannelType) -> ChannelType {
        match ct {
            channels::ChannelType::GithubRelease => ChannelType::GithubRelease,
            channels::ChannelType::RssFeed => ChannelType::RssFeed,
            channels::ChannelType::Twitter => ChannelType::Twitter,
        }
    }
}

impl Into<JsonResponse> for Channel {
    fn into(self) -> JsonResponse {
        match serde_json::to_value(self.clone()) {
            Ok(v) => JsonResponse::Ok(JsonValue(v)),
            Err(err) => {
                eprintln!(
                    "Failed to convert Channel {:?} into JsonResponse: {:?}",
                    self, err
                );
                JsonResponse::InternalServerError
            }
        }
    }
}

impl Into<JsonResponse> for Vec<Channel> {
    fn into(self) -> JsonResponse {
        match serde_json::to_value(self) {
            Ok(v) => JsonResponse::Ok(JsonValue(v)),
            Err(err) => {
                eprintln!(
                    "Failed to convert Vec<Channel> into JsonResponse: {:?}",
                    err
                );
                JsonResponse::InternalServerError
            }
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq)]
struct Channel {
    id: Option<i32>,
    name: String,
    summary: Option<String>,
    #[serde(rename = "type")]
    channel_type: ChannelType,
    creator: Option<String>,
    #[serde(rename = "creatorId")]
    creator_id: Option<i32>,
    link: Option<String>,
    verified: bool,
}

impl Channel {
    fn from_channel_info(c: ChannelInfo, ctype: channels::ChannelType) -> Channel {
        Self {
            id: None,
            name: c.name,
            summary: None,
            channel_type: ctype.into(),
            creator: None,
            creator_id: None,
            link: Some(c.link),
            verified: c.verified,
        }
    }

    fn from_channel(c: db::Channel) -> Channel {
        Self {
            id: Some(c.id),
            name: c.name,
            summary: None,
            channel_type: c.channel_type.into(),
            creator: None,
            creator_id: None,
            link: Some(c.link),
            verified: c.verified,
        }
    }

    fn from_list(l: db::List, i: db::Identity, channels: Vec<db::Channel>) -> Channel {
        Self {
            id: Some(l.id),
            name: l.name,
            channel_type: ChannelType::List,
            summary: Some(format!("{} channels", channels.len())),
            creator: Some(i.username),
            creator_id: Some(i.user_id),
            link: None,
            verified: false,
        }
    }
}

#[get("/<channel_type>/<id>")]
fn show(db: DigesterDbConn, channel_type: ChannelType, id: i32) -> JsonResponse {
    let maybe_channel = match channel_type {
        ChannelType::List => match db::lists_find_by_id(&db, id) {
            Ok(v) => v.map(|(l, i)| Channel::from_list(l, i, Vec::new())),
            Err(err) => {
                eprintln!("Failed to fetch list from db {:?}", err);
                return JsonResponse::InternalServerError;
            }
        },
        ChannelType::GithubRelease | ChannelType::RssFeed | ChannelType::Twitter => {
            match db::channels_find_by_id_opt(&db, id) {
                Ok(v) => v.map(Channel::from_channel),
                Err(err) => {
                    eprintln!("Failed to fetch channel from db {:?}", err);
                    return JsonResponse::InternalServerError;
                }
            }
        }
    };

    match maybe_channel {
        None => JsonResponse::NotFound,
        Some(c) => c.into(),
    }
}

#[get("/search?<channel_type>&<query>")]
fn search(
    _session: Protected,
    _r: RateLimited,
    db: DigesterDbConn,
    channel_type: ChannelType,
    gh_token: State<GithubApiToken>,
    query: &RawStr,
) -> JsonResponse {
    let query = match query.url_decode() {
        Ok(query) => query,
        Err(err) => {
            eprintln!("Failed to decode raw string: '{}': {:?}", query, err);
            return JsonResponse::InternalServerError;
        }
    };

    let channel_or_list = match channel_type {
        ChannelType::List => Left(()),
        ChannelType::GithubRelease => Right(db::ChannelType::GithubRelease),
        ChannelType::Twitter => Right(db::ChannelType::Twitter),
        ChannelType::RssFeed => Right(db::ChannelType::RssFeed),
    };

    match channel_or_list {
        Left(()) => {
            let lists = match db::lists_search(&db, &query) {
                Ok(lists) => lists,
                Err(err) => {
                    eprintln!("Failed to search for lists by '{}': {:?}", query, err);
                    return JsonResponse::InternalServerError;
                }
            };

            let lists_with_channels = match db::lists_identity_zip_with_channels(&db, lists) {
                Ok(lwc) => lwc,
                Err(err) => {
                    eprintln!("Failed to zip channels for lists: {}", err);
                    return JsonResponse::InternalServerError;
                }
            };

            lists_with_channels
                .into_iter()
                .map(|(list, id, channels)| Channel::from_list(list, id, channels))
                .collect::<Vec<Channel>>()
                .into()
        }
        Right(db_channel_type) => {
            let channel_type = match db_channel_type {
                db::ChannelType::GithubRelease => channels::ChannelType::GithubRelease,
                db::ChannelType::RssFeed => channels::ChannelType::RssFeed,
                db::ChannelType::Twitter => channels::ChannelType::Twitter,
            };
            // todo only initialize this if needed
            let github: GithubRelease = match GithubRelease::new(&gh_token.0) {
                Ok(github) => github,
                Err(err) => {
                    eprintln!("Failed to resolve github client: {:?}", err);
                    return JsonResponse::InternalServerError;
                }
            };

            let twitter: Twitter = match Twitter::new("") {
                Ok(twitter) => twitter,
                Err(err) => {
                    eprintln!("Failed to resolve twitter client: {:?}", err);
                    return JsonResponse::InternalServerError;
                }
            };
            let channel = channels::factory(&channel_type, &github, &twitter);

            let online_query = match channel.sanitize(&query) {
                Err(err) => {
                    eprintln!("Query is not a URL: {:?}", err);
                    return JsonResponse::BadRequest("Invalid Input".into());
                }
                Ok(oq) => oq,
            };

            match channel.search(online_query) {
                Err(SearchError::ChannelNotFound(msg)) => {
                    eprintln!("Channel not found: {}", msg);
                    JsonResponse::BadRequest(
                        "This does not exist. Are you sure the input is correct?".into(),
                    )
                }
                Err(SearchError::TechnicalError(msg)) => {
                    eprintln!("Technical error during search: {}", msg);
                    JsonResponse::InternalServerError
                }
                Err(SearchError::Timeout(msg)) => {
                    eprintln!("Timeout during online search: {}", msg);
                    JsonResponse::BadRequest(
                        "We could not fetch your feed fast enough. Please try again later.".into(),
                    )
                }
                Ok(channels) => channels
                    .into_iter()
                    .map(|c| Channel::from_channel_info(c, channel_type.clone()))
                    .collect::<Vec<Channel>>()
                    .into(),
            }
        }
    }
}
