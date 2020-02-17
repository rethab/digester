use lib_db as db;

use super::common::*;
use rocket::Rocket;
use rocket_contrib::json::{Json, JsonValue};

pub fn mount(rocket: Rocket) -> Rocket {
    rocket.mount("/lists", routes![list, get, add, update, delete])
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
    fn from_db(l: db::List, user: db::Identity) -> List {
        List {
            id: l.id,
            name: l.name,
            creator: user.username,
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

    lists
        .into_iter()
        .map(|(list, identity)| List::from_db(list, identity))
        .collect::<Vec<List>>()
        .into()
}

#[get("/<list_id>")]
fn get(db: DigesterDbConn, list_id: i32) -> JsonResponse {
    match db::lists_find_by_id(&db, list_id) {
        Ok(Some((list, identity))) => List::from_db(list, identity).into(),
        Ok(None) => JsonResponse::NotFound,
        Err(err) => {
            eprintln!("Failed to fetch list by id: {}", err);
            JsonResponse::InternalServerError
        }
    }
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
