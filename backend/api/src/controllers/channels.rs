use lib_channels as channels;
use lib_db as db;

use super::common::*;
use crate::iam::UserId;
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

pub struct TwitterTokens {
    pub api_key: String,
    pub api_secret_key: String,
    pub access_token: String,
    pub access_token_secret: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
enum ChannelType {
    RssFeed,
    GithubRelease,
    Twitter,
    List,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
struct Channel {
    id: Option<i32>,
    /// this, in combination with the channel_type uniquely identifies a channel
    /// ie. this must be unique in the specific channel (eg. a twitter handle or rss url)
    #[serde(rename = "extId")]
    ext_id: String,
    name: String,
    summary: Option<String>,
    #[serde(rename = "type")]
    channel_type: ChannelType,
    creator: Option<String>,
    #[serde(rename = "creatorId")]
    creator_id: Option<UserId>,
    link: Option<String>,
    verified: bool,
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
    twitter_tokens: State<TwitterTokens>,
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
            match online_search(&gh_token, &twitter_tokens, &query, channel_type) {
                Err(err) => err,
                Ok(channels) => {
                    if channels.is_empty() {
                        let no_channels: Vec<Channel> = Vec::new();
                        return JsonResponse::Ok(json!(no_channels));
                    }

                    let new_channels = channels
                        .iter()
                        .map(|f| db::NewChannel {
                            channel_type: db_channel_type,
                            name: f.name.clone(),
                            ext_id: f.ext_id.clone(),
                            link: f.link.clone(),
                            verified: f.verified,
                        })
                        .collect();

                    match db::channels_insert_many(&db, new_channels) {
                        Err(err) => {
                            eprintln!("Failed to insert new channels: {:?}", err);
                            JsonResponse::InternalServerError
                        }
                        Ok(db_channels) => JsonResponse::Ok(json!(db_channels
                            .into_iter()
                            .map(Channel::from_channel)
                            .collect::<Vec<Channel>>())),
                    }
                }
            }
        }
    }
}

fn online_search(
    gh_token: &GithubApiToken,
    twitter_tokens: &TwitterTokens,
    query: &str,
    channel_type: channels::ChannelType,
) -> Result<Vec<ChannelInfo>, JsonResponse> {
    // todo only initialize this if needed
    let github: GithubRelease = match GithubRelease::new(&gh_token.0) {
        Ok(github) => github,
        Err(err) => {
            eprintln!("Failed to resolve github client: {:?}", err);
            return Err(JsonResponse::InternalServerError);
        }
    };

    let twitter: Twitter = match Twitter::new(
        &twitter_tokens.api_key,
        &twitter_tokens.api_secret_key,
        &twitter_tokens.access_token,
        &twitter_tokens.access_token_secret,
    ) {
        Ok(twitter) => twitter,
        Err(err) => {
            eprintln!("Failed to resolve twitter client: {:?}", err);
            return Err(JsonResponse::InternalServerError);
        }
    };
    let channel = channels::factory(&channel_type, &github, &twitter);

    let online_query = match channel.sanitize(&query) {
        Err(err) => {
            eprintln!("Query is not a URL: {:?}", err);
            return Err(JsonResponse::BadRequest("Invalid Input".into()));
        }
        Ok(oq) => oq,
    };

    channel.search(online_query).map_err(|err| match err {
        SearchError::ChannelNotFound(msg) => {
            eprintln!("Channel not found: {}", msg);
            JsonResponse::BadRequest(
                "This does not exist. Are you sure the input is correct?".into(),
            )
        }
        SearchError::TechnicalError(msg) => {
            eprintln!("Technical error during search: {}", msg);
            JsonResponse::InternalServerError
        }
        SearchError::Timeout(msg) => {
            eprintln!("Timeout during online search: {}", msg);
            JsonResponse::BadRequest(
                "We could not fetch your feed fast enough. Please try again later.".into(),
            )
        }
    })
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

impl Channel {
    fn from_channel(c: db::Channel) -> Channel {
        Self {
            id: Some(c.id),
            ext_id: c.ext_id,
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
            ext_id: format!("{}", l.id),
            name: l.name,
            channel_type: ChannelType::List,
            summary: Some(format!("{} channels", channels.len())),
            creator: Some(i.username),
            creator_id: Some(UserId::from(i.user_id)),
            link: None,
            verified: false,
        }
    }
}
