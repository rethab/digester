use lib_db as db;

use super::super::iam::UserId;
use super::super::lists;
use super::common::*;
use db::ChannelType;
use rocket::http::RawStr;
use rocket::Rocket;
use rocket_contrib::json::{Json, JsonValue};

pub fn mount(rocket: Rocket) -> Rocket {
    rocket.mount(
        "/lists",
        routes![
            list,
            popular,
            show,
            search,
            add,
            update,
            add_channel,
            remove_channel,
            delete
        ],
    )
}

#[derive(Deserialize, Debug, PartialEq)]
struct NewList {
    name: String,
}

#[derive(Deserialize, Debug, PartialEq)]
struct AddChannel {
    id: i32,
}

#[derive(Deserialize, Debug, PartialEq)]
struct RemoveChannel {
    id: i32,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
struct List {
    id: i32,
    name: String,
    creator: String,
    #[serde(rename = "creatorId")]
    creator_id: UserId,
    channels: Vec<Channel>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
struct Channel {
    id: i32,
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
    fn from_db(l: db::List, identity: db::Identity, channels: Vec<db::Channel>) -> List {
        List {
            id: l.id,
            name: l.name,
            creator: identity.username,
            creator_id: UserId::from(identity.user_id),
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
            id: c.id,
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
    let lists = match db::lists_find(&db, maybe_creator_id.map(|c| c.into())) {
        Ok(lists) => lists,
        Err(err) => {
            eprintln!("Failed to fetch lists from db {:?}", err);
            return JsonResponse::InternalServerError;
        }
    };

    lists_to_resp(&db, lists).into()
}

#[get("/popular")]
fn popular(db: DigesterDbConn) -> JsonResponse {
    let lists = match db::lists_find_order_by_popularity(&db, 10) {
        Ok(lists) => lists,
        Err(err) => {
            eprintln!("Failed to fetch lists from db {:?}", err);
            return JsonResponse::InternalServerError;
        }
    };
    lists_to_resp(&db, lists).into()
}

#[get("/<id>")]
fn show(db: DigesterDbConn, id: i32) -> JsonResponse {
    let list = match db::lists_find_by_id(&db, id) {
        Ok(None) => return JsonResponse::NotFound,
        Ok(Some(list)) => list,
        Err(err) => {
            eprintln!("Failed to fetch lists from db {:?}", err);
            return JsonResponse::InternalServerError;
        }
    };

    let mut lists = Vec::with_capacity(1);
    lists.push(list);
    lists_to_resp(&db, lists)[0].clone().into()
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
    let lists_with_channels = match db::lists_identity_zip_with_channels(&db, lists) {
        Ok(lwc) => lwc,
        Err(err) => {
            eprintln!("Failed to zip channels for lists from db: {}", err);
            return Vec::new();
        }
    };

    lists_with_channels
        .into_iter()
        .map(|(list, identity, channels)| List::from_db(list, identity, channels))
        .collect::<Vec<List>>()
}

#[delete("/<list_id>")]
fn delete(session: Protected, db: DigesterDbConn, list_id: i32) -> JsonResponse {
    use lists::DeleteError::*;
    match lists::delete(&db, list_id, session.0.user_id) {
        Ok(()) => JsonResponse::Ok(json!({})),
        Err(OtherSubscriptions) => {
            JsonResponse::BadRequest("This list has other subscribers besides you".into())
        }
        Err(NotFound) => JsonResponse::NotFound,
        Err(Authorization) => JsonResponse::Forbidden,
        Err(Unknown(err)) => {
            eprintln!(
                "Failed to delete list {} for user {}: {}",
                list_id, session.0.user_id, err
            );
            JsonResponse::InternalServerError
        }
    }
}

#[put("/", data = "<new_list>")]
fn add(session: Protected, db: DigesterDbConn, new_list: Json<NewList>) -> JsonResponse {
    use lists::AddError::*;
    match lists::add(&db, session.0.user_id, new_list.name.clone()) {
        Err(InvalidName(msg)) => JsonResponse::BadRequest(msg),
        Err(UnknownError(msg)) => {
            eprintln!("Failed to add list: {}", msg);
            JsonResponse::InternalServerError
        }
        Ok((list, identity)) => {
            let mut lists = Vec::with_capacity(1);
            lists.push((list, identity));
            lists_to_resp(&db, lists)[0].clone().into()
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
    let list = match get_own_list(&db, session.0.user_id, list_id) {
        Ok(list) => list,
        Err(err) => return err,
    };

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

#[post("/<list_id>/add_channel", data = "<add_channel>")]
fn add_channel(
    session: Protected,
    db: DigesterDbConn,
    list_id: i32,
    add_channel: Json<AddChannel>,
) -> JsonResponse {
    let list = match get_own_list(&db, session.0.user_id, list_id) {
        Ok(list) => list,
        Err(err) => return err,
    };
    let channel_id = add_channel.id;
    match db::lists_add_channel(&db, list, channel_id) {
        Ok(()) => JsonResponse::Ok(json!("")),
        Err(err) => {
            eprintln!(
                "Failed to add channel {} to list {}: {:?}",
                channel_id, list_id, err
            );
            JsonResponse::InternalServerError
        }
    }
}

#[post("/<list_id>/remove_channel", data = "<remove_channel>")]
fn remove_channel(
    session: Protected,
    db: DigesterDbConn,
    list_id: i32,
    remove_channel: Json<RemoveChannel>,
) -> JsonResponse {
    let list = match get_own_list(&db, session.0.user_id, list_id) {
        Ok(list) => list,
        Err(err) => return err,
    };
    let channel_id = remove_channel.id;
    match db::lists_remove_channel(&db, list, channel_id) {
        Ok(()) => JsonResponse::Ok(json!("")),
        Err(err) => {
            eprintln!(
                "Failed to remove channel {} from list {}: {:?}",
                channel_id, list_id, err
            );
            JsonResponse::InternalServerError
        }
    }
}

fn get_own_list(
    db: &DigesterDbConn,
    user_id: UserId,
    list_id: i32,
) -> Result<db::List, JsonResponse> {
    use lists::Error::*;
    lists::get_own_list(&db, user_id, list_id).map_err(|err| match err {
        NotFound => JsonResponse::NotFound,
        Authorization => JsonResponse::Forbidden,
        Unknown(err) => {
            eprintln!(
                "Failed to load list {} for user {}: {}",
                list_id, user_id, err
            );
            JsonResponse::InternalServerError
        }
    })
}
