#[macro_use]
extern crate diesel;

use chrono::{DateTime, Duration, Utc};
use chrono_tz::Tz;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result;
use std::env;

pub struct Connection(PgConnection);

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

pub fn channels_find_by_id(conn: &Connection, channel_id: i32) -> Result<Channel, String> {
    use schema::channels::dsl::*;
    channels
        .find(channel_id)
        .get_result(&conn.0)
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

pub enum InsertError {
    Duplicate,
    Unknown,
}

impl InsertError {
    fn from_diesel(err: diesel::result::Error) -> InsertError {
        if InsertError::is_unique_constrait_violation(err) {
            InsertError::Duplicate
        } else {
            InsertError::Unknown
        }
    }

    fn is_unique_constrait_violation(error: diesel::result::Error) -> bool {
        match error {
            result::Error::DatabaseError(kind, _) => match kind {
                result::DatabaseErrorKind::UniqueViolation => true,
                _ => false,
            },
            _ => false,
        }
    }
}

pub fn channels_insert_if_not_exists(
    conn: &PgConnection,
    new_channel: NewChannel,
) -> Result<Channel, String> {
    use schema::channels::dsl::*;

    let find = || -> Result<Option<Channel>, String> {
        channels
            .filter(
                name.eq(&new_channel.name)
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
                        None => Err("Found no channel after duplicate insert error".to_owned()),
                    })
                }
                Err(InsertError::Unknown) => Err("Failed to insert new channel".to_owned()),
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

pub fn updates_insert_new(conn: &Connection, update: &NewUpdate) -> Result<(), InsertError> {
    use schema::updates;

    diesel::insert_into(updates::table)
        .values(update)
        .execute(&conn.0)
        .map_err(InsertError::from_diesel)
        .map(|_| ())
}

pub fn updates_find_new(
    conn: &Connection,
    subscription: &Subscription,
    since: DateTime<Utc>,
) -> Result<Vec<Update>, String> {
    use schema::updates::dsl::*;
    updates
        .filter(
            channel_id
                .eq(subscription.channel_id)
                .and(published.gt(since)),
        )
        .load(&conn.0)
        .map_err(|err| format!("Failed to load updates: {:?}", err))
}

pub fn updates_find_by_user_id(
    conn: &PgConnection,
    user_id: i32,
    offset: u32,
    limit: u32,
) -> Result<Vec<(Update, Channel)>, String> {
    use schema::channels;
    use schema::subscriptions;
    use schema::updates;

    subscriptions::table
        .inner_join(channels::table.on(subscriptions::channel_id.eq(channels::id)))
        .inner_join(updates::table.on(channels::id.eq(updates::channel_id)))
        .filter(subscriptions::user_id.eq(user_id))
        .order_by(updates::published.desc())
        .offset(offset as i64)
        .limit(limit as i64)
        .select((updates::all_columns, channels::all_columns))
        .load::<(Update, Channel)>(conn)
        .map_err(|err| format!("Failed to load updates by id: {:?}", err))
}

pub fn subscriptions_find_by_id(
    conn: &PgConnection,
    id: i32,
    user_id: i32,
) -> Result<Option<(Subscription, Channel)>, String> {
    use schema::channels;
    use schema::subscriptions;
    subscriptions::table
        .inner_join(channels::table.on(subscriptions::channel_id.eq(channels::id)))
        .filter(
            subscriptions::id
                .eq(id)
                .and(subscriptions::user_id.eq(user_id)),
        )
        .select((subscriptions::all_columns, channels::all_columns))
        .load::<(Subscription, Channel)>(conn)
        .map(|subs| subs.into_iter().next())
        .map_err(|err| format!("Failed to load subscription by id: {:?}", err))
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
) -> Result<Vec<(Subscription, Channel)>, String> {
    use schema::channels;
    use schema::subscriptions;
    subscriptions::table
        .inner_join(channels::table.on(subscriptions::channel_id.eq(channels::id)))
        .filter(subscriptions::user_id.eq(user_id))
        .select((subscriptions::all_columns, channels::all_columns))
        .load::<(Subscription, Channel)>(conn)
        .map_err(|err| {
            format!(
                "Failed to run query in subscriptions_find_without_due_digests: {:?}",
                err
            )
        })
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
        .inner_join(users::table.on(subscriptions::user_id.eq(users::id)))
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
    use schema::identities;
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
    let identity: Identity = diesel::insert_into(identities::table)
        .values(new_identity)
        .returning(identities::all_columns)
        .get_result(conn)
        .map_err(|err| format!("Failed to insert new identity: {:?}", err))?;

    Ok((user, identity))
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
