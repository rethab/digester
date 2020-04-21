#[macro_use]
extern crate diesel;

use chrono::{DateTime, Duration, Utc};
use chrono_tz::Tz;
use diesel::dsl::sql;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result;
use diesel::result::Error;
use diesel::sql_types::BigInt;
use either::{Either, Left, Right};
use std::collections::HashMap;
use std::env;
use std::iter::FromIterator;

pub struct Connection(pub PgConnection);

pub type RichSubscription = (Subscription, Either<Channel, List>);

pub mod model;
mod schema;

pub use model::*;

pub fn connection_from_env() -> Result<Connection, String> {
    let connection_string = env::var("DATABASE_CONNECTION")
        .map_err(|_err| "Missing connection string in env variable".to_owned())?;
    connection_from_str(&connection_string)
}

pub fn connection_from_str(uri: &str) -> Result<Connection, String> {
    diesel::connection::Connection::establish(uri)
        .map_err(|err| format!("Failed to connect to database: {:?}", err))
        .map(Connection)
}

pub fn channels_find_by_id(conn: &PgConnection, channel_id: i32) -> Result<Channel, String> {
    use schema::channels::dsl::*;
    channels
        .find(channel_id)
        .get_result(conn)
        .map_err(|err| format!("Failed to fetch channel with id {}: {:?}", channel_id, err))
}

pub fn channels_find_by_id_opt(
    conn: &PgConnection,
    channel_id: i32,
) -> Result<Option<Channel>, String> {
    use schema::channels::dsl::*;
    channels
        .find(channel_id)
        .load::<Channel>(conn)
        .map(|cs| {
            if cs.len() == 1 {
                Some(cs[0].clone())
            } else {
                None
            }
        })
        .map_err(|err| format!("Failed to fetch channel with id {}: {:?}", channel_id, err))
}

pub fn channels_find_by_last_fetched(
    conn: &Connection,
    fetch_frequency: Duration,
) -> Result<Vec<Channel>, String> {
    use schema::channels::dsl::*;

    let since_last_fetched = Utc::now() - fetch_frequency;

    channels
        .filter(
            last_fetched
                .lt(since_last_fetched)
                .or(last_fetched.is_null()),
        )
        .load::<Channel>(&conn.0)
        .map_err(|err| {
            format!(
                "Failed to run query in channels_find_by_last_fetched: {:?}",
                err
            )
        })
}

pub fn channels_find_by_last_cleaned(
    conn: &Connection,
    clean_frequency: Duration,
) -> Result<Vec<Channel>, String> {
    use schema::channels::dsl::*;

    let since_last_cleaned = Utc::now() - clean_frequency;

    channels
        .filter(
            last_cleaned
                .lt(since_last_cleaned)
                .or(last_cleaned.is_null()),
        )
        .load::<Channel>(&conn.0)
        .map_err(|err| {
            format!(
                "Failed to run query in channels_find_by_last_cleaned: {:?}",
                err
            )
        })
}

pub fn channels_find_by_list_id(conn: &PgConnection, list_id: i32) -> Result<Vec<Channel>, String> {
    use schema::channels;
    use schema::lists_channels;

    channels::table
        .inner_join(lists_channels::table.on(lists_channels::channel_id.eq(channels::id)))
        .filter(lists_channels::list_id.eq(list_id))
        .select(channels::all_columns)
        .load::<Channel>(conn)
        .map_err(|err| format!("Failed to load channels by list id {}: {:?}", list_id, err))
}

pub enum InsertError {
    Duplicate,
    Unknown(diesel::result::Error),
}

impl InsertError {
    fn from_diesel(err: diesel::result::Error) -> InsertError {
        if InsertError::is_unique_constrait_violation(&err) {
            InsertError::Duplicate
        } else {
            InsertError::Unknown(err)
        }
    }

    fn is_unique_constrait_violation(error: &diesel::result::Error) -> bool {
        match error {
            result::Error::DatabaseError(kind, _) => match kind {
                result::DatabaseErrorKind::UniqueViolation => true,
                _ => false,
            },
            _ => false,
        }
    }
}

pub fn channels_insert_many(
    conn: &PgConnection,
    channels: Vec<NewChannel>,
) -> Result<Vec<Channel>, String> {
    // todo ideally this would be an insert_many sql statement
    let mut db_channels = Vec::with_capacity(channels.len());
    for channel in channels {
        let new_channels = channels_insert_if_not_exists(conn, channel)?;
        db_channels.push(new_channels);
    }
    Ok(db_channels)
}

pub fn channels_insert_if_not_exists(
    conn: &PgConnection,
    new_channel: NewChannel,
) -> Result<Channel, String> {
    use schema::channels::dsl::*;

    let find = || -> Result<Option<Channel>, String> {
        channels
            .filter(
                ext_id
                    .eq(&new_channel.ext_id)
                    .and(channel_type.eq(&new_channel.channel_type)),
            )
            .load(conn)
            .map_err(|err| format!("Failed to query for channel: {:?}", err))
            .and_then(|results| {
                if results.len() > 1 {
                    Err(format!(
                        "Found more than one channel for name {} and type {:?}",
                        new_channel.name, new_channel.channel_type
                    ))
                } else {
                    Ok(results.into_iter().next())
                }
            })
    };

    match find() {
        Err(err) => Err(err),
        Ok(Some(channel)) => Ok(channel),
        Ok(None) => {
            use schema::channels;
            match diesel::insert_into(channels::table)
                .values(&new_channel)
                .returning(channels::all_columns)
                .get_result(conn)
                .map_err(InsertError::from_diesel)
            {
                Ok(channel) => Ok(channel),
                Err(InsertError::Duplicate) => {
                    find().and_then(|maybe_channel| match maybe_channel {
                        Some(channel) => Ok(channel),
                        None => Err(format!(
                            "Not found again after duplicate insert error: {:?}",
                            new_channel
                        )),
                    })
                }
                Err(InsertError::Unknown(err)) => {
                    Err(format!("Failed to insert new channel: {:?}", err))
                }
            }
        }
    }
}

pub fn channels_update_last_fetched(conn: &Connection, channel: &Channel) -> Result<(), String> {
    use diesel::expression::dsl::now;
    use schema::channels::dsl::*;
    diesel::update(channels.find(channel.id))
        .set(last_fetched.eq(now))
        .execute(&conn.0)
        .map_err(|err| {
            format!(
                "failed to update last_fetched field for channel {}: {:?}",
                channel.id, err
            )
        })
        .map(|_| ())
}

pub fn channels_update_last_cleaned_by_ids(
    conn: &Connection,
    channel_ids: Vec<i32>,
) -> Result<(), String> {
    use diesel::expression::dsl::now;
    use schema::channels::dsl::*;
    diesel::update(channels.filter(id.eq_any(channel_ids.clone())))
        .set(last_cleaned.eq(now))
        .execute(&conn.0)
        .map_err(|err| {
            format!(
                "failed to update last_cleaned field for channels {:?}: {:?}",
                channel_ids, err
            )
        })
        .map(|_| ())
}

pub fn updates_insert_new(conn: &Connection, update: &NewUpdate) -> Result<(), InsertError> {
    use schema::updates;

    diesel::insert_into(updates::table)
        .values(update)
        .execute(&conn.0)
        .map_err(InsertError::from_diesel)
        .map(|_| ())
}

pub fn updates_find_newest_by_channel(
    conn: &Connection,
    channel_id: i32,
) -> Result<Option<Update>, String> {
    use schema::updates;
    updates::table
        .filter(updates::channel_id.eq(channel_id))
        .order_by(updates::inserted.desc())
        .limit(1)
        .load(&conn.0)
        .map(|us| us.into_iter().next())
        .map_err(|err| {
            format!(
                "Failed to find newest update by channel {}: {:?}",
                channel_id, err
            )
        })
}

pub fn updates_find_new(
    conn: &Connection,
    chan_id: i32,
    published_or_inserted_since: Either<DateTime<Utc>, DateTime<Utc>>,
) -> Result<Vec<Update>, String> {
    use schema::updates::dsl::*;
    let query = match published_or_inserted_since {
        Left(p) => updates
            .filter(channel_id.eq(chan_id).and(published.gt(p)))
            .load(&conn.0),
        Right(i) => updates
            .filter(channel_id.eq(chan_id).and(inserted.gt(i)))
            .load(&conn.0),
    };
    query.map_err(|err| format!("Failed to load updates: {:?}", err))
}

pub fn updates_find_ext_ids_by_channel_ids(
    conn: &Connection,
    channel_ids: &[i32],
) -> Result<HashMap<i64, String>, String> {
    use schema::updates;
    updates::table
        .filter(updates::channel_id.eq_any(channel_ids))
        .select((updates::id, updates::ext_id))
        .load::<(i64, Option<String>)>(&conn.0)
        .map(|us| {
            HashMap::from_iter(
                us.into_iter()
                    .flat_map(|(id, maybe_ext_id)| maybe_ext_id.map(|ext_id| (id, ext_id))),
            )
        })
        .map_err(|err| {
            format!(
                "Failed to fetch updates by channel_id {:?}: {:?}",
                channel_ids, err
            )
        })
}

pub fn updates_find_by_user_id(
    conn: &PgConnection,
    user_id: i32,
    offset: u32,
    limit: u32,
) -> Result<Vec<(Update, Channel)>, String> {
    let mut channel_ids = Vec::new();
    for (_, chan_or_list) in subscriptions_find_by_user_id(conn, user_id)? {
        match chan_or_list {
            Left(chan) => channel_ids.push(chan.id),
            Right(list) => {
                for channel in channels_find_by_list_id(conn, list.id)? {
                    channel_ids.push(channel.id);
                }
            }
        }
    }

    use schema::channels;
    use schema::updates;
    channels::table
        .inner_join(updates::table.on(channels::id.eq(updates::channel_id)))
        .filter(channels::id.eq_any(channel_ids))
        .order_by(updates::published.desc())
        .offset(offset as i64)
        .limit(limit as i64)
        .select((updates::all_columns, channels::all_columns))
        .load::<(Update, Channel)>(conn)
        .map_err(|err| format!("Failed to load updates by id: {:?}", err))
}

pub fn updates_delete_old_by_channel_id(
    conn: &Connection,
    channel_id: i32,
    retain_updates_duration: Duration,
) -> Result<usize, String> {
    use schema::updates;
    // need to run two queries, because subqueries with the same table are
    // not supported in diesel: https://github.com/diesel-rs/diesel/issues/1369

    let delete_before = Utc::now() - retain_updates_duration;
    let ids_to_delete = updates::table
        .filter(
            updates::inserted
                .lt(delete_before)
                .and(updates::channel_id.eq(channel_id)),
        )
        .select(updates::id)
        .order_by(updates::inserted.desc())
        .offset(1) // need to retain one to compare against when fetching updates
        .load::<i64>(&conn.0)
        .map_err(|err| {
            format!(
                "Failed to fetch updates before {:?} for channel_id {}: {:?}",
                retain_updates_duration, channel_id, err
            )
        })?;
    updates_delete_by_ids(conn, ids_to_delete)
}

pub fn updates_delete_by_ids(conn: &Connection, ids: Vec<i64>) -> Result<usize, String> {
    use schema::updates;
    diesel::delete(updates::table)
        .filter(updates::id.eq_any(ids.clone()))
        .execute(&conn.0)
        .map_err(|err| format!("Failed to delete update ids {:?}: {:?}", ids, err))
}

pub fn subscriptions_find_by_id_user_id(
    conn: &PgConnection,
    id: i32,
    user_id: i32,
) -> Result<Option<RichSubscription>, String> {
    use schema::subscriptions;

    let mb_sub = subscriptions::table
        .filter(
            subscriptions::id
                .eq(id)
                .and(subscriptions::user_id.eq(user_id)),
        )
        .first(conn)
        .optional()
        .map_err(|err| format!("Failed to load subscription by id: {:?}", err))?;

    let sub = match mb_sub {
        Some(sub) => sub,
        None => return Ok(None),
    };

    Ok(Some(subscriptions_zip_with_channel_or_list(conn, sub)?))
}

pub fn subscriptions_find_by_digest(
    conn: &Connection,
    digest: &Digest,
) -> Result<Subscription, String> {
    use schema::subscriptions::dsl::*;

    subscriptions
        .find(digest.subscription_id)
        .first(&conn.0)
        .map_err(|err| format!("Failed to run query: {:?}", err))
}

pub fn subscriptions_find_without_due_digest(
    conn: &Connection,
) -> Result<Vec<Subscription>, String> {
    use schema::digests;
    use schema::subscriptions;
    subscriptions::table
        .left_join(
            digests::table.on(digests::sent
                .is_null()
                .and(digests::subscription_id.eq(subscriptions::id))),
        )
        .filter(digests::subscription_id.is_null())
        .select(subscriptions::all_columns)
        .load::<Subscription>(&conn.0)
        .map_err(|err| {
            format!(
                "Failed to run query in subscriptions_find_without_due_digests: {:?}",
                err
            )
        })
}

pub fn subscriptions_find_by_user_id(
    conn: &PgConnection,
    user_id: i32,
) -> Result<Vec<RichSubscription>, String> {
    use schema::subscriptions;

    let subscriptions = subscriptions::table
        .filter(subscriptions::user_id.eq(user_id))
        .select(subscriptions::all_columns)
        .load::<Subscription>(conn)
        .map_err(|err| {
            format!(
                "Failed to fetch subscriptions for user {}: {:?}",
                user_id, err
            )
        })?;

    let mut results = Vec::new();
    for sub in subscriptions {
        results.push(subscriptions_zip_with_channel_or_list(conn, sub)?);
    }
    Ok(results)
}

pub fn subscriptions_find_by_list_id(
    conn: &PgConnection,
    list_id: i32,
) -> Result<Vec<Subscription>, String> {
    use schema::subscriptions;
    subscriptions::table
        .filter(subscriptions::list_id.eq(list_id))
        .select(subscriptions::all_columns)
        .load::<Subscription>(conn)
        .map_err(|err| {
            format!(
                "Failed to fetch subscriptions for list_id {}: {:?}",
                list_id, err
            )
        })
}

fn subscriptions_zip_with_channel_or_list(
    conn: &PgConnection,
    sub: Subscription,
) -> Result<(Subscription, Either<Channel, List>), String> {
    match (sub.channel_id, sub.list_id) {
        (Some(channel_id), None) => {
            let channel = channels_find_by_id(conn, channel_id)?;
            Ok((sub, Left(channel)))
        }
        (None, Some(list_id)) => match lists_find_by_id(conn, list_id)? {
            Some((list, _)) => Ok((sub, Right(list))),
            None => Err(format!(
                "Failed to fetch list {} for subscription {}",
                list_id, sub.id
            )),
        },
        _ => Err(format!(
            "Subscription {} has channel_id and list_id or neither",
            sub.id
        )),
    }
}

pub fn subscriptions_insert(
    conn: &PgConnection,
    sub: NewSubscription,
) -> Result<Subscription, InsertError> {
    use schema::subscriptions;

    diesel::insert_into(subscriptions::table)
        .values(&sub)
        .returning(subscriptions::all_columns)
        .get_result(conn)
        .map_err(InsertError::from_diesel)
}

pub fn subscriptions_update(
    conn: &PgConnection,
    sub: Subscription,
) -> Result<Subscription, String> {
    sub.save_changes(conn)
        .map_err(|err| format!("Failed to update single subscription: {:?}", err))
}

pub fn subscriptions_delete_by_user_id(conn: &PgConnection, user_id: i32) -> Result<(), Error> {
    // note that this can fail if we are creating digests at the same time
    use schema::digests;
    use schema::subscriptions;

    let subs_ids_query = subscriptions::table
        .filter(subscriptions::user_id.eq(user_id))
        .select(subscriptions::id);

    diesel::delete(digests::table.filter(digests::subscription_id.eq_any(subs_ids_query)))
        .execute(conn)?;

    diesel::delete(subscriptions::table.filter(subscriptions::user_id.eq(user_id)))
        .execute(conn)
        .map(|_| ())
}

pub fn subscriptions_delete_by_id(conn: &PgConnection, sub_id: i32) -> Result<(), Error> {
    use schema::subscriptions;
    diesel::delete(subscriptions::table.filter(subscriptions::id.eq(sub_id)))
        .execute(conn)
        .map(|_| ())
}

pub fn pending_subscriptions_insert(
    conn: &PgConnection,
    sub: NewPendingSubscription,
) -> Result<PendingSubscription, InsertError> {
    use schema::pending_subscriptions;
    diesel::insert_into(pending_subscriptions::table)
        .values(sub)
        .returning(pending_subscriptions::all_columns)
        .get_result(conn)
        .map_err(InsertError::from_diesel)
}

pub fn pending_subscriptions_set_sent(
    conn: &PgConnection,
    pending_subscription: &PendingSubscription,
    sent: DateTime<Utc>,
) -> Result<(), String> {
    use schema::pending_subscriptions::dsl::*;
    diesel::update(pending_subscriptions.find(id))
        .set(activation_email_sent.eq(sent))
        .execute(conn)
        .map(|_| ())
        .map_err(|err| {
            format!(
                "Failed to update 'activation_email_sent' for pending_subscription {}: {:?}",
                pending_subscription.id, err
            )
        })
}

pub fn pending_subscriptions_find_by_token(
    db: &PgConnection,
    token: &str,
) -> Result<Option<PendingSubscription>, String> {
    use schema::pending_subscriptions;
    pending_subscriptions::table
        .filter(pending_subscriptions::token.eq(token))
        .get_results::<PendingSubscription>(db)
        .map(|xs| {
            if xs.len() == 1 {
                Some(xs[0].clone())
            } else {
                None
            }
        })
        .map_err(|err| format!("Failed to fetch pending subscription by token: {:?}", err))
}

pub fn pending_subscriptions_delete(
    db: &PgConnection,
    pending_sub: PendingSubscription,
) -> Result<(), String> {
    use schema::pending_subscriptions::dsl::*;
    diesel::delete(pending_subscriptions.filter(id.eq(pending_sub.id)))
        .execute(db)
        .map(|_| ())
        .map_err(|err| format!("Failed pending subscription {:?}", err))
}

pub fn digests_insert(conn: &Connection, digest: &InsertDigest) -> Result<(), InsertError> {
    use schema::digests;
    diesel::insert_into(digests::table)
        .values(digest)
        .execute(&conn.0)
        .map_err(InsertError::from_diesel)
        .map(|_| ())
}

pub fn digests_find_users_with_due(conn: &Connection) -> Result<Vec<User>, String> {
    use diesel::expression::dsl::now;
    use schema::digests;
    use schema::subscriptions;
    use schema::users;
    digests::table
        .inner_join(subscriptions::table.on(digests::subscription_id.eq(subscriptions::id)))
        .inner_join(users::table.on(subscriptions::user_id.eq(users::id.nullable())))
        .filter(digests::due.lt(now).and(digests::sent.is_null()))
        .distinct()
        .select(users::all_columns)
        .load::<User>(&conn.0)
        .map_err(|err| format!("failed to retrieve users with due digests: {:?}", err))
}

pub fn digests_find_due_for_user(
    conn: &Connection,
    user: &User,
) -> Result<Vec<(Digest, Subscription)>, String> {
    use diesel::expression::dsl::now;
    use schema::digests;
    use schema::subscriptions;
    digests::table
        .inner_join(subscriptions::table.on(digests::subscription_id.eq(subscriptions::id)))
        .filter(
            digests::due
                .lt(now)
                .and(digests::sent.is_null())
                .and(subscriptions::user_id.eq(user.id)),
        )
        .select((digests::all_columns, subscriptions::all_columns))
        .load::<(Digest, Subscription)>(&conn.0)
        .map_err(|err| format!("failed to retrieve due digests for user: {:?}", err))
}
pub fn digests_find_previous(conn: &Connection, digest: &Digest) -> Result<Option<Digest>, String> {
    use schema::digests::dsl::*;
    digests
        .filter(
            subscription_id
                .eq(digest.subscription_id)
                .and(sent.is_not_null()),
        )
        .order_by(sent.desc())
        .limit(1)
        .first(&conn.0)
        .optional()
        .map_err(|err| format!("Failed to find previous digest: {:?}", err))
}

pub fn digests_set_sent(conn: &Connection, digest: &Digest) -> Result<(), String> {
    use diesel::expression::dsl::now;
    use schema::digests::dsl::*;
    diesel::update(digests.find(digest.id))
        .set(sent.eq(now))
        .execute(&conn.0)
        .map(|_| ())
        .map_err(|err| format!("Failed to update 'sent' for digest {:?}: {:?}", digest, err))
}

pub fn digests_remove_unsent_for_subscription(
    conn: &PgConnection,
    sub: &Subscription,
) -> Result<(), String> {
    use schema::digests::dsl::*;
    diesel::delete(digests.filter(subscription_id.eq(sub.id).and(sent.is_null())))
        .execute(conn)
        .map(|_| ())
        .map_err(|err| format!("Failed remove digest for subscription {:?}", err))
}

pub fn digests_remove_unsent_for_user(conn: &PgConnection, user: &User) -> Result<(), String> {
    let subs = subscriptions_find_by_user_id(conn, user.id)?;
    for (sub, _) in subs {
        digests_remove_unsent_for_subscription(conn, &sub)?;
    }
    Ok(())
}

pub fn digests_delete_by_subscription_id(conn: &PgConnection, sub_id: i32) -> Result<(), Error> {
    use schema::digests::dsl::*;
    diesel::delete(digests.filter(subscription_id.eq(sub_id)))
        .execute(conn)
        .map(|_| ())
}

pub fn users_find_by_provider(
    conn: &PgConnection,
    provider: &str,
    pid: &str,
) -> Result<Option<(User, Identity)>, String> {
    use schema::identities;
    use schema::users;

    users::table
        .inner_join(identities::table.on(identities::user_id.eq(users::id)))
        .filter(
            identities::provider
                .eq(provider)
                .and(identities::pid.eq(pid)),
        )
        .select((users::all_columns, identities::all_columns))
        .load::<(User, Identity)>(conn)
        .map_err(|err| format!("Failed to fetch user by provider: {}", err))
        .and_then(|uis| {
            if uis.len() > 1 {
                Err(format!(
                    "Found more than one entry for provider={} and pid={}",
                    provider, pid
                ))
            } else {
                Ok(uis.into_iter().next())
            }
        })
}

pub fn users_find_by_username(conn: &PgConnection, username: &str) -> Result<User, String> {
    use schema::identities;
    use schema::users;

    users::table
        .inner_join(identities::table.on(identities::user_id.eq(users::id)))
        .filter(identities::username.eq(username))
        .select(users::all_columns)
        .get_result::<User>(conn)
        .map_err(|err| format!("Failed to fetch user username {}: {}", username, err))
}

pub fn users_update_timezone(conn: &PgConnection, user_id: i32, new_tz: Tz) -> Result<(), String> {
    use schema::users::dsl::*;
    diesel::update(users.find(user_id))
        .set(timezone.eq(Timezone(new_tz)))
        .execute(conn)
        .map_err(|err| format!("failed to update timezone of user {}: {:?}", user_id, err))
        .map(|_| ())
}

pub fn users_find_by_id(conn: &PgConnection, user_id: i32) -> Result<User, String> {
    use schema::users::dsl::*;
    users
        .find(user_id)
        .get_result(conn)
        .map_err(|err| format!("Failed to fetch user {}: {:?}", user_id, err))
}

pub fn users_find_by_id0(conn: &Connection, user_id: i32) -> Result<User, String> {
    users_find_by_id(&conn.0, user_id)
}

pub struct NewUserData {
    pub provider: String,
    pub pid: String,
    pub email: String,
    pub username: String,
}

pub fn users_insert(
    conn: &PgConnection,
    new_user: NewUserData,
) -> Result<(User, Identity), String> {
    use schema::users;

    let user: User = diesel::insert_into(users::table)
        .default_values()
        .returning(users::all_columns)
        .get_result(conn)
        .map_err(|err| format!("Failed to insert new user: {:?}", err))?;

    let new_identity = NewIdentity {
        provider: new_user.provider,
        pid: new_user.pid,
        user_id: user.id,
        email: new_user.email,
        username: new_user.username,
    };
    let identity = identities_insert(conn, new_identity)?;

    Ok((user, identity))
}

pub fn users_delete_by_id(conn: &PgConnection, user_id: i32) -> Result<(), Error> {
    use schema::identities;
    use schema::users;

    diesel::delete(identities::table.filter(identities::user_id.eq(user_id))).execute(conn)?;

    diesel::delete(users::table.filter(users::id.eq(user_id)))
        .execute(conn)
        .map(|_| ())
}

pub fn identities_find_by_user_id(
    conn: &PgConnection,
    id_of_user: i32,
) -> Result<Identity, String> {
    use schema::identities::dsl::*;
    identities
        .filter(user_id.eq(id_of_user))
        .get_result(conn)
        .map_err(|err| {
            format!(
                "Failed to fetch identity for user_id {}: {:?}",
                id_of_user, err
            )
        })
}

pub fn identities_find_by_user_ids(
    conn: &PgConnection,
    ids_of_users: &[i32],
) -> Result<Vec<Identity>, String> {
    use schema::identities::dsl::*;
    identities
        .filter(user_id.eq_any(ids_of_users))
        .get_results(conn)
        .map_err(|err| {
            format!(
                "Failed to fetch identity for user_id {:?}: {:?}",
                ids_of_users, err
            )
        })
}

pub fn identities_find_by_email_or_id(
    conn: &PgConnection,
    provider: &str,
    pid: &str,
    email: &str,
) -> Result<Vec<Identity>, String> {
    use schema::identities;

    identities::table
        .filter(
            identities::email.eq(email).or(identities::provider
                .eq(provider)
                .and(identities::pid.eq(pid))),
        )
        .load(conn)
        .map_err(|err| {
            format!(
                "Failed to query for identities by provider={}, pid={}, email={}: {:?}",
                provider, pid, email, err
            )
        })
}

pub fn identities_update_email(
    conn: &PgConnection,
    identity: Identity,
    new_email: &str,
) -> Result<Identity, String> {
    use schema::identities::dsl::*;
    diesel::update(identities.find(identity.id))
        .set(email.eq(new_email))
        .returning(identities::all_columns())
        .get_result(conn)
        .map_err(|err| {
            format!(
                "failed to update email of identity={} {:?}",
                identity.id, err
            )
        })
}

pub fn identities_insert(conn: &PgConnection, identity: NewIdentity) -> Result<Identity, String> {
    use schema::identities;
    diesel::insert_into(identities::table)
        .values(identity)
        .returning(identities::all_columns)
        .get_result(conn)
        .map_err(|err| format!("Failed to insert new identity: {:?}", err))
}

// finds lists and their creators. Note that if the creator
// has two identities, the first one is picked at random
fn lists_zip_with_identities(
    conn: &PgConnection,
    lists: Vec<List>,
) -> Result<Vec<(List, Identity)>, String> {
    let user_ids = lists.iter().map(|l: &List| l.creator).collect::<Vec<i32>>();
    let identities = identities_find_by_user_ids(conn, &user_ids).map_err(|err| {
        format!(
            "Failed to fetch identities for user ids: {:?}: {:?}",
            user_ids, err
        )
    })?;

    let mut results: Vec<(List, Identity)> = Vec::with_capacity(lists.len());
    for list in lists {
        match find_identity(&identities, list.creator) {
            Some(identity) => results.push((list, identity)),
            None => eprintln!(
                "Found no identity for list {} with creator id {}",
                list.id, list.creator
            ),
        }
    }
    Ok(results)
}

fn find_identity(identities: &[Identity], user_id: i32) -> Option<Identity> {
    for identity in identities {
        if identity.user_id == user_id {
            return Some(identity.clone());
        }
    }
    None
}

pub fn lists_find(
    conn: &PgConnection,
    creator_id: Option<i32>,
) -> Result<Vec<(List, Identity)>, String> {
    use schema::lists::dsl::*;
    let results = match creator_id {
        Some(user_id) => lists.filter(creator.eq(user_id)).get_results(conn),
        None => lists.get_results(conn),
    };
    lists_zip_with_identities(
        conn,
        results.map_err(|err| format!("Failed to query for lists: {:?}", err))?,
    )
}

pub fn lists_search(conn: &PgConnection, query: &str) -> Result<Vec<(List, Identity)>, String> {
    use schema::lists::dsl::*;

    let search_query = format!("%{}%", query);
    let results = lists
        .filter(name.ilike(search_query))
        .get_results(conn)
        .map_err(|err| format!("Failed to fetch channels by query: {:?}", err))?;

    lists_zip_with_identities(conn, results)
}

pub fn lists_find_by_id(
    conn: &PgConnection,
    list_id: i32,
) -> Result<Option<(List, Identity)>, String> {
    use schema::lists::dsl::*;
    let ls = lists
        .find(list_id)
        .get_results(conn)
        .map_err(|err| format!("Failed to query for lists: {:?}", err))?;

    let list_and_identity = lists_zip_with_identities(conn, ls)?;
    if list_and_identity.len() == 1 {
        Ok(list_and_identity.into_iter().next())
    } else if list_and_identity.is_empty() {
        Ok(None)
    } else {
        Err(format!(
            "Searching lists by id {} returned more than one result: {:?}",
            list_id, list_and_identity
        ))
    }
}

pub fn lists_find_by_user_id(conn: &PgConnection, user_id: i32) -> Result<Vec<List>, Error> {
    use schema::lists::dsl::*;
    lists.filter(creator.eq(user_id)).get_results(conn)
}

pub fn lists_find_with_other_subscribers(
    conn: &PgConnection,
    user_id: i32,
) -> Result<Vec<List>, Error> {
    use schema::lists;
    use schema::subscriptions;
    lists::table
        .inner_join(subscriptions::table.on(subscriptions::list_id.eq(lists::id.nullable())))
        .filter(
            lists::creator
                .eq(user_id)
                .and(subscriptions::user_id.ne(user_id)),
        )
        .select(lists::all_columns)
        .load::<List>(conn)
}

pub fn lists_find_order_by_popularity(
    conn: &PgConnection,
    limit: i32,
) -> Result<Vec<(List, Identity)>, String> {
    use schema::lists;
    use schema::subscriptions;

    // see diesel issue: https://github.com/diesel-rs/diesel/issues/210
    let query = lists::table
        .select((lists::all_columns, sql::<BigInt>("sum(1) as cnt")))
        .left_join(subscriptions::table.on(subscriptions::list_id.eq(lists::id.nullable())))
        .limit(limit as i64)
        .group_by(lists::id)
        .order_by(sql::<BigInt>("cnt").desc());

    let ls: Vec<List> = query
        .load::<(List, i64)>(conn)
        .map(|pairs| pairs.into_iter().map(|(l, _)| l).collect::<Vec<List>>())
        .map_err(|err| format!("Failed to load lists ordered by popularity: {:?}", err))?;

    lists_zip_with_identities(conn, ls)
}

pub fn lists_delete_by_id(conn: &PgConnection, list_id: i32) -> Result<(), Error> {
    use schema::lists;
    use schema::lists_channels;

    // delete all links to channels
    diesel::delete(lists_channels::table.filter(lists_channels::list_id.eq(list_id)))
        .execute(conn)
        .map(|_| ())?;

    // delete list itself
    diesel::delete(lists::table.filter(lists::id.eq(list_id)))
        .execute(conn)
        .map(|_| ())
}

pub fn lists_insert(conn: &PgConnection, new_list: &NewList) -> Result<List, String> {
    use schema::lists;
    diesel::insert_into(lists::table)
        .values(new_list)
        .returning(lists::all_columns)
        .get_result(conn)
        .map_err(|err| format!("Failed to insert new list {:?}: {:?}", new_list, err))
}

pub fn lists_update_name(conn: &PgConnection, list: List) -> Result<List, String> {
    list.save_changes(conn)
        .map_err(|err| format!("Failed to update list: {:?}", err))
}

pub fn lists_move_creator(
    conn: &PgConnection,
    list_ids: &[i32],
    new_creator: i32,
) -> Result<(), Error> {
    use schema::lists::dsl::*;
    diesel::update(lists.filter(id.eq_any(list_ids)))
        .set(creator.eq(new_creator))
        .execute(conn)
        .map(|_| ())
}

pub fn lists_add_channel(conn: &PgConnection, list: List, channel_id: i32) -> Result<(), String> {
    use schema::lists_channels;

    let list_channel = NewListChannel {
        list_id: list.id,
        channel_id,
    };
    diesel::insert_into(lists_channels::table)
        .values(list_channel)
        .execute(conn)
        .map(|_| ())
        .or_else(|err| match InsertError::from_diesel(err) {
            InsertError::Duplicate => Ok(()),
            InsertError::Unknown(err) => Err(format!("Failed to insert: {:?}", err)),
        })
}

pub fn lists_remove_channel(
    conn: &PgConnection,
    list: List,
    channel_id: i32,
) -> Result<(), String> {
    use schema::lists_channels;
    diesel::delete(
        lists_channels::table.filter(
            lists_channels::list_id
                .eq(list.id)
                .and(lists_channels::channel_id.eq(channel_id)),
        ),
    )
    .execute(conn)
    .map(|_| ())
    .map_err(|err| {
        format!(
            "Failed to delete list {} channel {} mapping: {:?}",
            list.id, channel_id, err
        )
    })
}

pub fn lists_identity_zip_with_channels(
    conn: &PgConnection,
    lists: Vec<(List, Identity)>,
) -> Result<Vec<(List, Identity, Vec<Channel>)>, String> {
    let mut lists_with_channels = Vec::with_capacity(lists.len());
    for (list, identity) in lists {
        let channels = channels_find_by_list_id(conn, list.id)?;
        lists_with_channels.push((list, identity, channels));
    }
    Ok(lists_with_channels)
}

pub fn lists_zip_with_channels(
    conn: &PgConnection,
    lists: Vec<List>,
) -> Result<Vec<(List, Vec<Channel>)>, String> {
    let mut lists_with_channels = Vec::with_capacity(lists.len());
    for list in lists {
        let channels = channels_find_by_list_id(conn, list.id)?;
        lists_with_channels.push((list, channels));
    }
    Ok(lists_with_channels)
}
