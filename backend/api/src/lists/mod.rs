use super::iam::UserId;
use super::subscriptions;
use chrono::naive::NaiveTime;
use diesel::pg::PgConnection;
use lib_db as db;

pub enum DeleteError {
    OtherSubscriptions,
    NotFound,
    Authorization,
    Unknown(String),
}

impl From<Error> for DeleteError {
    fn from(e: Error) -> DeleteError {
        match e {
            Error::NotFound => DeleteError::NotFound,
            Error::Authorization => DeleteError::Authorization,
            Error::Unknown(msg) => DeleteError::Unknown(msg),
        }
    }
}

pub enum Error {
    NotFound,
    Authorization,
    Unknown(String),
}

pub fn delete(db: &PgConnection, list_id: i32, user_id: UserId) -> Result<(), DeleteError> {
    // ensure list exists and is owned by active user
    let list = get_own_list(&db, user_id, list_id)?;
    println!("Deleting list with id {} for user_id {}", list.id, user_id);
    if let Ok(subscriptions) = db::subscriptions_find_by_list_id(&db, list.id) {
        if subscriptions.iter().any(|s| s.user_id != Some(user_id.0)) {
            return Err(DeleteError::OtherSubscriptions);
        } else {
            for sub in subscriptions {
                if let Err(err) = subscriptions::delete(&db, sub.id) {
                    return Err(DeleteError::Unknown(format!(
                        "Failed to delete subscriptions {}: {}",
                        sub.id, err
                    )));
                }
            }
        }
    }
    match db::lists_delete_by_id(&db, list_id) {
        Ok(()) => Ok(()),
        Err(err) => Err(DeleteError::Unknown(format!(
            "Failed to delete list by id: {}",
            err
        ))),
    }
}

pub enum AddError {
    InvalidName(String),
    UnknownError(String),
}

pub fn add(
    db: &PgConnection,
    user_id: UserId,
    mut list_name: String,
) -> Result<(db::List, db::Identity), AddError> {
    list_name = list_name.trim().into();
    if list_name.len() < 5 || list_name.len() > 30 {
        return Err(AddError::InvalidName(
            "Name must be between 5 and 30 characters".into(),
        ));
    }

    let new_list = db::NewList {
        name: list_name,
        creator: user_id.0,
    };
    match db::lists_insert(&db, &new_list) {
        Ok(list) => {
            if let Err(err) = subscribe_user(&db, user_id, &list) {
                eprintln!(
                    "Failed to subscribe user {} after creating list {}: {}",
                    user_id, list.id, err
                )
            }
            db::identities_find_by_user_id(&db, list.creator)
                .map_err(|err| {
                    AddError::UnknownError(format!(
                        "Creator {} for list {} not found in DB: {}",
                        list.creator, list.id, err
                    ))
                })
                .map(|identity| (list, identity))
        }
        Err(err) => Err(AddError::UnknownError(format!(
            "Failed to insert new list {:?}: {:?}",
            new_list, err
        ))),
    }
}

fn subscribe_user(db: &PgConnection, user_id: UserId, list: &db::List) -> Result<(), String> {
    use db::InsertError::*;
    let identity = db::identities_find_by_user_id(&db, user_id.0)?;
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

pub fn get_own_list(db: &PgConnection, user_id: UserId, list_id: i32) -> Result<db::List, Error> {
    let list = match db::lists_find_by_id(&db, list_id) {
        Ok(Some((list, _))) => list,
        Ok(None) => return Err(Error::NotFound),
        Err(err) => {
            return Err(Error::Unknown(format!(
                "Failed to fetch list with id {}: {}",
                list_id, err
            )))
        }
    };

    if list.creator != user_id.0 {
        Err(Error::Authorization)
    } else {
        Ok(list)
    }
}
