#[macro_use]
extern crate diesel;

use chrono::{DateTime, Duration, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result;
use std::env;

pub struct Connection(PgConnection);

pub use models::*;

mod schema {

    table! {
        users(id) {
            id -> Integer,
        }
    }

    table! {
        identities(id) {
            id -> Integer,
            provider -> Text,
            pid -> Text,
            user_id -> Integer,
            email -> Text,
            username -> Text,
        }
    }

    table! {
        blogs (id) {
            id -> Integer,
            url -> Text,
            last_fetched -> Nullable<Timestamptz>,
        }
    }

    table! {
        posts (id) {
            id -> BigInt,
            blog_id -> Integer,
            title -> Text,
            author -> Nullable<Text>,
            url -> Text,
            published -> Timestamptz,
            inserted -> Timestamptz,
        }
    }

    table! {
        subscriptions(id) {
          id -> Integer,
          email -> Text,
          blog_id -> Integer,
          frequency -> Text,
          day -> Nullable<Text>,
          time -> Time,
        }
    }

    table! {
        digests(id) {
          id -> BigInt,
          subscription_id -> Integer,
          due -> Timestamptz,
          sent -> Nullable<Timestamptz>,
        }
    }

    allow_tables_to_appear_in_same_query!(subscriptions, digests);
    allow_tables_to_appear_in_same_query!(users, identities);
}

mod models {
    use super::schema::*;
    use chrono::naive::NaiveTime;
    use chrono::{DateTime, Utc};
    use diesel::deserialize::{self, FromSql};
    use diesel::pg::Pg;
    use diesel::serialize::{self, IsNull, Output, ToSql};
    use diesel::sql_types::Text;
    use diesel::*;
    use std::io::Write;

    #[derive(Queryable)]
    pub struct Identity {
        pub id: i32,
        pub provider: String,
        pub pid: String,
        pub user_id: i32,
        pub email: String,
        pub username: String,
    }

    #[derive(Insertable)]
    #[table_name = "identities"]
    pub struct NewIdentity {
        pub provider: String,
        pub pid: String,
        pub user_id: i32,
        pub email: String,
        pub username: String,
    }

    #[derive(Queryable)]
    pub struct User {
        pub id: i32,
    }

    #[derive(Queryable)]
    pub struct Blog {
        pub id: i32,
        pub url: String,
        pub last_fetched: Option<DateTime<Utc>>,
    }

    #[derive(Insertable)]
    #[table_name = "blogs"]
    pub struct NewBlog {
        pub url: String,
    }

    #[derive(Debug, Insertable)]
    #[table_name = "posts"]
    pub struct NewPost {
        pub blog_id: i32,
        pub title: String,
        pub author: Option<String>,
        pub url: String,
        pub published: DateTime<Utc>,
        pub inserted: DateTime<Utc>,
    }

    #[derive(Debug, Queryable)]
    pub struct Post {
        pub id: i64,
        pub blog_id: i32,
        pub title: String,
        pub author: Option<String>,
        pub url: String,
        pub published: DateTime<Utc>,
        pub inserted: DateTime<Utc>,
    }

    #[derive(Debug, PartialEq, FromSqlRow, AsExpression)]
    #[sql_type = "Text"]
    pub enum Frequency {
        Daily,
        Weekly,
    }

    #[derive(Debug, PartialEq, FromSqlRow, AsExpression)]
    #[sql_type = "Text"]
    pub enum Day {
        Mon,
        Tue,
        Wed,
        Thu,
        Fri,
        Sat,
        Sun,
    }

    impl Day {
        pub fn to_weekday(&self) -> chrono::Weekday {
            match self {
                Day::Mon => chrono::Weekday::Mon,
                Day::Tue => chrono::Weekday::Tue,
                Day::Wed => chrono::Weekday::Wed,
                Day::Thu => chrono::Weekday::Thu,
                Day::Fri => chrono::Weekday::Fri,
                Day::Sat => chrono::Weekday::Sat,
                Day::Sun => chrono::Weekday::Sun,
            }
        }
    }

    #[derive(Debug, Queryable)]
    pub struct Subscription {
        pub id: i32,
        pub email: String,
        pub blog_id: i32,
        pub frequency: Frequency,
        pub day: Option<Day>,
        pub time: NaiveTime,
    }

    #[derive(Insertable, Debug)]
    #[table_name = "digests"]
    pub struct InsertDigest {
        pub subscription_id: i32,
        pub due: DateTime<Utc>,
    }

    #[derive(Debug, Queryable)]
    pub struct Digest {
        pub id: i64,
        pub subscription_id: i32,
        pub due: DateTime<Utc>,
        pub sent: Option<DateTime<Utc>>,
    }

    impl ToSql<Text, Pg> for Frequency {
        fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
            match *self {
                Frequency::Daily => out.write_all(b"daily")?,
                Frequency::Weekly => out.write_all(b"weekly")?,
            }
            Ok(IsNull::No)
        }
    }

    impl FromSql<Text, Pg> for Frequency {
        fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
            match not_none!(bytes) {
                b"daily" => Ok(Frequency::Daily),
                b"weekly" => Ok(Frequency::Weekly),
                unrecognized => {
                    Err(format!("Unrecognized frequency enum variant: {:?}", unrecognized).into())
                }
            }
        }
    }

    impl FromSql<Text, Pg> for Day {
        fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
            match not_none!(bytes) {
                b"mon" => Ok(Day::Mon),
                b"tue" => Ok(Day::Tue),
                b"wed" => Ok(Day::Wed),
                b"thu" => Ok(Day::Thu),
                b"fri" => Ok(Day::Fri),
                b"sat" => Ok(Day::Sat),
                b"sun" => Ok(Day::Sun),
                unrecognized => {
                    Err(format!("Unrecognized day enum variant: {:?}", unrecognized).into())
                }
            }
        }
    }

    impl ToSql<Text, Pg> for Day {
        fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
            match *self {
                Day::Mon => out.write_all(b"mon")?,
                Day::Tue => out.write_all(b"tue")?,
                Day::Wed => out.write_all(b"wed")?,
                Day::Thu => out.write_all(b"thu")?,
                Day::Fri => out.write_all(b"fri")?,
                Day::Sat => out.write_all(b"sat")?,
                Day::Sun => out.write_all(b"sun")?,
            }
            Ok(IsNull::No)
        }
    }
}

pub fn connection_from_env() -> Result<Connection, String> {
    let connection_string = env::var("DATABASE_CONNECTION")
        .map_err(|_err| "Missing connection string in env variable".to_owned())?;
    diesel::connection::Connection::establish(connection_string.as_str())
        .map_err(|err| format!("Failed to connect to database: {:?}", err))
        .map(Connection)
}

pub fn blogs_find_by_last_fetched(
    conn: &Connection,
    fetch_frequency: Duration,
) -> Result<Vec<Blog>, String> {
    use schema::blogs::dsl::*;

    let since_last_fetched = Utc::now() - fetch_frequency;

    blogs
        .filter(
            last_fetched
                .lt(since_last_fetched)
                .or(last_fetched.is_null()),
        )
        .load::<Blog>(&conn.0)
        .map_err(|err| {
            format!(
                "Failed to run query in blogs_find_by_last_fetched: {:?}",
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

pub fn blogs_insert(conn: &PgConnection, blog: NewBlog) -> Result<(), InsertError> {
    use schema::blogs;

    diesel::insert_into(blogs::table)
        .values(&blog)
        .execute(conn)
        .map(|_| ())
        .map_err(InsertError::from_diesel)
}

pub fn blogs_update_last_fetched(conn: &Connection, blog: &Blog) -> Result<(), String> {
    use diesel::expression::dsl::now;
    use schema::blogs::dsl::*;
    diesel::update(blogs.find(blog.id))
        .set(last_fetched.eq(now))
        .execute(&conn.0)
        .map_err(|err| {
            format!(
                "failed to update last_fetched field for blog {}: {:?}",
                blog.id, err
            )
        })
        .map(|_| ())
}

pub fn posts_insert_new(conn: &Connection, post: &NewPost) -> Result<(), InsertError> {
    use schema::posts;

    diesel::insert_into(posts::table)
        .values(post)
        .execute(&conn.0)
        .map_err(InsertError::from_diesel)
        .map(|_| ())
}

pub fn posts_find_new(
    conn: &Connection,
    subscription: &Subscription,
    maybe_previous_digest_sent: Option<DateTime<Utc>>,
) -> Result<Vec<Post>, String> {
    use schema::posts::dsl::*;
    let result = if let Some(previous_digest_sent) = maybe_previous_digest_sent {
        posts
            .filter(
                blog_id
                    .eq(subscription.blog_id)
                    .and(inserted.gt(previous_digest_sent)),
            )
            .load(&conn.0)
    } else {
        posts.filter(blog_id.eq(subscription.blog_id)).load(&conn.0)
    };
    result.map_err(|err| format!("Failed to load posts: {:?}", err))
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
pub fn digests_insert(conn: &Connection, digest: &InsertDigest) -> Result<(), InsertError> {
    use schema::digests;
    diesel::insert_into(digests::table)
        .values(digest)
        .execute(&conn.0)
        .map_err(InsertError::from_diesel)
        .map(|_| ())
}

pub fn digests_find_due(conn: &Connection) -> Result<Vec<Digest>, String> {
    use diesel::expression::dsl::now;
    use schema::digests::dsl::*;
    digests
        .filter(due.lt(now).and(sent.is_null()))
        .load::<Digest>(&conn.0)
        .map_err(|err| format!("failed to retrieve due digests: {:?}", err))
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
