use super::iam::UserId;
use chrono::naive::NaiveTime;
use db::{Day, Frequency};
use diesel::pg::PgConnection;
use either::{Either, Left, Right};
use lib_db as db;

pub fn delete(db: &PgConnection, id: i32) -> Result<(), String> {
    db.build_transaction()
        .run(|| {
            db::digests_delete_by_subscription_id(db, id)?;
            db::subscriptions_delete_by_id(db, id)
        })
        .map_err(|err| format!("Failed to delete subscriptions and user {}: {:?}", id, err,))
}

pub enum SearchChannelType {
    List(i32),
    Channel(i32),
}

pub enum AddError {
    Unknown(String),
    NotFound(String),
    AlreadyExists,
}

type RichSubscription = (
    db::Subscription,
    Either<db::Channel, (db::List, Vec<db::Channel>)>,
);

pub fn add(
    user_id: UserId,
    db: &PgConnection,
    channel_type: SearchChannelType,
    frequency: Frequency,
    day: Option<Day>,
    time: NaiveTime,
) -> Result<RichSubscription, AddError> {
    use AddError::*;

    let identity = db::identities_find_by_user_id(&db, user_id.0).map_err(Unknown)?;

    match channel_type {
        SearchChannelType::List(list_id) => {
            let list = db::lists_find_by_id(&db, list_id)
                .map_err(Unknown)
                .and_then(|maybe_list| match maybe_list {
                    Some((list, _)) => Ok(list),
                    None => Err(NotFound("list does not exist".into())),
                })?;

            let new_subscription = db::NewSubscription {
                email: identity.email.clone(),
                timezone: None,
                channel_id: None,
                list_id: Some(list.id),
                user_id: Some(identity.user_id),
                frequency,
                day,
                time,
            };
            let sub = db::subscriptions_insert(&db, new_subscription).map_err(|err| match err {
                db::InsertError::Duplicate => AlreadyExists,
                db::InsertError::Unknown(err) => {
                    Unknown(format!("Failed to list channel subscription: {:?}", err))
                }
            })?;
            let channels = db::channels_find_by_list_id(&db, list.id).map_err(|err| {
                Unknown(format!(
                    "Failed to find channels by list id {}: {:?}",
                    list.id, err
                ))
            })?;
            Ok((sub, Right((list, channels))))
        }
        SearchChannelType::Channel(channel_id) => {
            let channel = db::channels_find_by_id(&db, channel_id)
                .map_err(|_| NotFound("channel does not exist".into()))?;

            let new_subscription = db::NewSubscription {
                email: identity.email.clone(),
                timezone: None,
                channel_id: Some(channel.id),
                list_id: None,
                user_id: Some(identity.user_id),
                frequency,
                day,
                time,
            };
            let sub = db::subscriptions_insert(&db, new_subscription).map_err(|err| match err {
                db::InsertError::Duplicate => AlreadyExists,
                db::InsertError::Unknown(err) => {
                    Unknown(format!("Failed to insert channel subscription: {:?}", err))
                }
            })?;
            Ok((sub, Left(channel)))
        }
    }
}
