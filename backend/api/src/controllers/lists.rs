use lib_db as db;

use super::common::*;
use chrono::naive::NaiveTime;
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
    creator_id: i32,
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
    fn from_db(l: db::List, user: db::Identity, channels: Vec<db::Channel>) -> List {
        List {
            id: l.id,
            name: l.name,
            creator: user.username,
            creator_id: user.id,
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
    let lists = match db::lists_find(&db, maybe_creator_id) {
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
    // ensure list exists and is owned by active user
    if let Err(err) = get_own_list(&db, &session, list_id) {
        return err;
    };

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
        Ok(list) => {
            let user_id = session.0.user_id;
            if let Err(err) = subscribe_user(&db, user_id, &list) {
                eprintln!(
                    "Failed to subscribe user {} after creating list {}: {}",
                    user_id, list.id, err
                )
            }
            let identity = match db::identities_find_by_user_id(&db, list.creator) {
                Err(err) => {
                    eprintln!(
                        "Creator {} for list {} not found in DB: {}",
                        list.creator, list.id, err
                    );
                    return JsonResponse::InternalServerError;
                }
                Ok(id) => id,
            };

            let mut lists = Vec::with_capacity(1);
            lists.push((list, identity));
            lists_to_resp(&db, lists)[0].clone().into()
        }
        Err(err) => {
            eprintln!("Failed to insert new list {:?}: {:?}", new_list, err);
            JsonResponse::InternalServerError
        }
    }
}

fn subscribe_user(db: &DigesterDbConn, user_id: i32, list: &db::List) -> Result<(), String> {
    use db::InsertError::*;
    let identity = db::identities_find_by_user_id(&db, user_id)?;
    let new_sub = db::NewSubscription {
        email: identity.email,
        timezone: None,
        channel_id: None,
        list_id: Some(list.id),
        user_id: Some(identity.user_id),
        frequency: db::Frequency::Weekly,
        day: Some(db::Day::Sat),
        time: NaiveTime::from_hms(9, 0, 0),
    };
    db::subscriptions_insert(&db, new_sub).map_err(|err| match err {
        Unknown(err) => format!("{:?}", err),
        Duplicate => "Duplicate".to_owned(),
    })?;
    Ok(())
}

#[post("/<list_id>", data = "<updated_list>")]
fn update(
    session: Protected,
    db: DigesterDbConn,
    list_id: i32,
    updated_list: Json<NewList>,
) -> JsonResponse {
    let list = match get_own_list(&db, &session, list_id) {
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
    let list = match get_own_list(&db, &session, list_id) {
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
    let list = match get_own_list(&db, &session, list_id) {
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
    session: &Protected,
    list_id: i32,
) -> Result<db::List, JsonResponse> {
    let list = match db::lists_find_by_id(&db, list_id) {
        Ok(Some((list, _))) => list,
        Ok(None) => return Err(JsonResponse::NotFound),
        Err(err) => {
            eprintln!("Failed to fetch list with id {}: {}", list_id, err);
            return Err(JsonResponse::InternalServerError);
        }
    };

    if list.creator != session.0.user_id {
        Err(JsonResponse::Forbidden)
    } else {
        Ok(list)
    }
}
