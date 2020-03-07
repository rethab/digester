use lib_db as db;

use super::common::*;
use rocket::Rocket;
use rocket_contrib::json::JsonValue;

pub fn mount(rocket: Rocket) -> Rocket {
    rocket.mount("/channels", routes![show,])
}

#[derive(Serialize, Clone, Debug, PartialEq)]
enum ChannelType {
    RssFeed,
    GithubRelease,
    List,
}

impl ChannelType {
    fn from_db(ct: db::ChannelType) -> ChannelType {
        match ct {
            db::ChannelType::GithubRelease => ChannelType::GithubRelease,
            db::ChannelType::RssFeed => ChannelType::RssFeed,
        }
    }

    fn from_request(s: &str) -> Result<ChannelType, String> {
        match s.to_ascii_lowercase().as_str() {
            "rssfeed" => Ok(ChannelType::RssFeed),
            "githubrelease" => Ok(ChannelType::GithubRelease),
            "list" => Ok(ChannelType::List),
            other => Err(format!("Invalid channel type: {}", other)),
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

#[derive(Serialize, Clone, Debug, PartialEq)]
struct Channel {
    id: i32,
    name: String,
    #[serde(rename = "type")]
    channel_type: ChannelType,
    creator: Option<String>,
    #[serde(rename = "creatorId")]
    creator_id: Option<i32>,
    link: Option<String>,
}

impl Channel {
    fn from_db_channel(c: db::Channel) -> Channel {
        Channel {
            id: c.id,
            name: c.name,
            channel_type: ChannelType::from_db(c.channel_type),
            creator: None,
            creator_id: None,
            link: Some(c.link),
        }
    }
    fn from_db_list(l: db::List, i: db::Identity) -> Channel {
        Channel {
            id: l.id,
            name: l.name,
            channel_type: ChannelType::List,
            creator: Some(i.username),
            creator_id: Some(i.user_id),
            link: None,
        }
    }
}

#[get("/<channel>/<id>")]
fn show(db: DigesterDbConn, channel: String, id: i32) -> JsonResponse {
    let channel_type = match ChannelType::from_request(&channel) {
        Ok(t) => t,
        Err(err) => {
            eprintln!("Failed to parse channel type: {}", err);
            return JsonResponse::BadRequest("Invalid channel type".to_string());
        }
    };
    let maybe_channel = match channel_type {
        ChannelType::List => match db::lists_find_by_id(&db, id) {
            Ok(v) => v.map(|(l, i)| Channel::from_db_list(l, i)),
            Err(err) => {
                eprintln!("Failed to fetch list from db {:?}", err);
                return JsonResponse::InternalServerError;
            }
        },
        ChannelType::GithubRelease | ChannelType::RssFeed => {
            match db::channels_find_by_id_opt(&db, id) {
                Ok(v) => v.map(Channel::from_db_channel),
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
