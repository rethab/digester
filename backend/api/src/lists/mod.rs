use super::iam::UserId;
use super::subscriptions;
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
