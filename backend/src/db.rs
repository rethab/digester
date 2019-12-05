use chrono::{Duration, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result;

pub struct Connection(PgConnection);

pub use models::*;

mod schema {
    table! {
        blogs (id) {
            id -> Integer,
            url -> Text,
            last_fetched -> Nullable<Timestamptz>,
        }
    }

    table! {
        posts (id) {
            id -> Integer,
            blog_id -> Integer,
            title -> Text,
            author -> Nullable<Text>,
            url -> Text,
            published -> Timestamptz,
            inserted -> Timestamptz,
        }
    }
}

mod models {
    use super::schema::*;
    use chrono::{DateTime, Utc};

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

    #[derive(Insertable)]
    #[table_name = "posts"]
    pub struct Post {
        pub blog_id: i32,
        pub title: String,
        pub author: Option<String>,
        pub url: String,
        pub published: DateTime<Utc>,
        pub inserted: DateTime<Utc>,
    }
}

pub fn connect(connection_string: &str) -> Result<Connection, String> {
    // todo we want TLS

    diesel::connection::Connection::establish(connection_string)
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

pub fn posts_insert_new(conn: &Connection, post: &Post) -> Result<(), InsertError> {
    use schema::posts;

    diesel::insert_into(posts::table)
        .values(post)
        .execute(&conn.0)
        .map_err(InsertError::from_diesel)
        .map(|_| ())
}
