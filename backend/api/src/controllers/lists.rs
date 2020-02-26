use lib_db as db;

use super::common::*;
use db::ChannelType;
use rocket::http::RawStr;
use rocket::Rocket;
use rocket_contrib::json::{Json, JsonValue};

pub fn mount(rocket: Rocket) -> Rocket {
    rocket.mount("/lists", routes![list, search, add, update, delete])
}

#[derive(Deserialize, Debug, PartialEq)]
struct NewList {
    name: String,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
struct List {
    id: i32,
    name: String,
    creator: String,
    channels: Vec<Channel>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
struct Channel {
    name: String,
    #[serde(rename = "type")]
    channel_type: ChannelType,
    link: String,
}

impl Into<JsonResponse> for List {
    fn into(self) -> JsonResponse {
        match serde_json::to_value(self.clone()) {
            Ok(v) => JsonResponse::Ok(JsonValue(v)),
            Err(err) => {
                eprintln!(
                    "Failed to convert List {:?} into JsonResponse: {:?}",
                    self, err
                );
                JsonResponse::InternalServerError
            }
        }
    }
}

impl Into<JsonResponse> for Vec<List> {
    fn into(self) -> JsonResponse {
        match serde_json::to_value(self) {
            Ok(v) => JsonResponse::Ok(JsonValue(v)),
            Err(err) => {
                eprintln!("Failed to convert Vec<List> into JsonResponse: {:?}", err);
                JsonResponse::InternalServerError
            }
        }
    }
}

impl List {
    fn from_db(l: db::List, user: db::Identity, channels: Vec<db::Channel>) -> List {
        List {
            id: l.id,
            name: l.name,
            creator: user.username,
            channels: channels
                .into_iter()
                .map(Channel::from_db)
                .collect::<Vec<Channel>>(),
        }
    }
}

impl Channel {
    fn from_db(c: db::Channel) -> Channel {
        Channel {
            name: c.name,
            channel_type: c.channel_type,
            link: c.link,
        }
    }
}

#[get("/?<own>")]
fn list(session: Option<Protected>, db: DigesterDbConn, own: Option<bool>) -> JsonResponse {
    let maybe_creator_id = match own {
        Some(true) => match session {
            Some(session) => Some(session.0.user_id),
            None => return JsonResponse::BadRequest("own parameter requires session".into()),
        },
        _ => None,
    };
    let lists = match db::lists_find(&db, maybe_creator_id) {
        Ok(lists) => lists,
        Err(err) => {
            eprintln!("Failed to fetch lists from db {:?}", err);
            return JsonResponse::InternalServerError;
        }
    };

    lists_to_resp(&db, lists).into()
}

#[get("/search?<query>")]
fn search(db: DigesterDbConn, query: &RawStr) -> JsonResponse {
    let lists = match db::lists_search(&db, query) {
        Ok(lists) => lists,
        Err(err) => {
            eprintln!("Failed to search for lists by '{}': {:?}", query, err);
            return JsonResponse::InternalServerError;
        }
    };

    lists_to_resp(&db, lists).into()
}

fn lists_to_resp(db: &DigesterDbConn, lists: Vec<(db::List, db::Identity)>) -> Vec<List> {
    let mut lists_with_channels = Vec::with_capacity(lists.len());
    for (list, identity) in lists {
        match db::channels_find_by_list_id(&db, list.id) {
            Ok(channels) => lists_with_channels.push((list, identity, channels)),
            Err(err) => eprintln!(
                "Failed to fetch channels for list {} from db: {:?}",
                list.id, err
            ),
        }
    }

    lists_with_channels
        .into_iter()
        .map(|(list, identity, channels)| List::from_db(list, identity, channels))
        .collect::<Vec<List>>()
}

#[delete("/<list_id>")]
fn delete(session: Protected, db: DigesterDbConn, list_id: i32) -> JsonResponse {
    let list = match db::lists_find_by_id(&db, list_id) {
        Ok(Some((list, _))) => list,
        Ok(None) => return JsonResponse::NotFound,
        Err(err) => {
            eprintln!(
                "Failed to fetch list with id {} for deletion: {}",
                list_id, err
            );
            return JsonResponse::InternalServerError;
        }
    };

    if list.creator != session.0.user_id {
        return JsonResponse::Forbidden;
    }

    println!(
        "Deleting list with id {} for user_id {}",
        list_id, session.0.user_id
    );
    match db::lists_delete_by_id(&db, list_id) {
        Ok(()) => JsonResponse::Ok(json!("")),
        Err(err) => {
            eprintln!("Failed to delete list by id: {}", err);
            JsonResponse::InternalServerError
        }
    }
}

#[put("/", data = "<new_list>")]
fn add(session: Protected, db: DigesterDbConn, new_list: Json<NewList>) -> JsonResponse {
    let mut list_name = new_list.name.clone();
    list_name = list_name.trim().into();
    if list_name.len() < 5 || list_name.len() > 30 {
        return JsonResponse::BadRequest("Name must be between 5 and 30 characters".into());
    }

    let new_list = db::NewList {
        name: list_name,
        creator: session.0.user_id,
    };
    match db::lists_insert(&db, &new_list) {
        Ok(list) => JsonResponse::Ok(json!({"id": list.id})),
        Err(err) => {
            eprintln!("Failed to insert new list {:?}: {:?}", new_list, err);
            JsonResponse::InternalServerError
        }
    }
}

#[post("/<list_id>", data = "<updated_list>")]
fn update(
    session: Protected,
    db: DigesterDbConn,
    list_id: i32,
    updated_list: Json<NewList>,
) -> JsonResponse {
    let list = match db::lists_find_by_id(&db, list_id) {
        Ok(Some((list, _))) => list,
        Ok(None) => return JsonResponse::NotFound,
        Err(err) => {
            eprintln!(
                "Failed to fetch list with id {} for deletion: {}",
                list_id, err
            );
            return JsonResponse::InternalServerError;
        }
    };

    if list.creator != session.0.user_id {
        return JsonResponse::Forbidden;
    }

    let mut list_name = updated_list.name.clone();
    list_name = list_name.trim().into();
    if list_name.len() < 5 || list_name.len() > 30 {
        return JsonResponse::BadRequest("Name must be between 5 and 30 characters".into());
    }

    let to_update = db::List {
        id: list_id,
        name: list_name,
        ..list
    };
    match db::lists_update_name(&db, to_update) {
        Ok(_) => JsonResponse::Ok(json!("")),
        Err(err) => {
            eprintln!("Failed to update list {}: {:?}", list_id, err);
            JsonResponse::InternalServerError
        }
    }
}
