use super::schema::*;
use chrono::naive::NaiveTime;
use chrono::{DateTime, Utc};
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::Text;
use diesel::*;
use serde::{Deserialize, Serialize};
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
pub struct Channel {
    pub id: i32,
    pub type_: ChannelType,
    pub name: String,
    pub last_fetched: Option<DateTime<Utc>>,
    pub inserted: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[table_name = "channels"]
pub struct NewChannel {
    pub type_: ChannelType,
    pub name: String,
}

#[derive(Debug, Insertable)]
#[table_name = "updates"]
pub struct NewUpdate {
    pub channel_id: i32,
    pub title: String,
    pub url: String,
    pub published: DateTime<Utc>,
}

#[derive(Debug, Queryable)]
pub struct Update {
    pub id: i64,
    pub channel_id: i32,
    pub title: String,
    pub url: String,
    pub published: DateTime<Utc>,
    pub inserted: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, FromSqlRow, AsExpression, Serialize, Deserialize)]
#[sql_type = "Text"]
pub enum ChannelType {
    GithubRelease,
}

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Serialize, Deserialize)]
#[sql_type = "Text"]
pub enum Frequency {
    Daily,
    Weekly,
}

#[derive(Debug, Clone, PartialEq, FromSqlRow, AsExpression, Serialize, Deserialize)]
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

impl Into<chrono::Weekday> for Day {
    fn into(self) -> chrono::Weekday {
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
    pub channel_id: i32,
    pub frequency: Frequency,
    pub day: Option<Day>,
    pub time: NaiveTime,
}

#[derive(Insertable)]
#[table_name = "subscriptions"]
pub struct NewSubscription {
    pub email: String,
    pub channel_id: i32,
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

impl ToSql<Text, Pg> for ChannelType {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            ChannelType::GithubRelease => out.write_all(b"github_release")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<Text, Pg> for ChannelType {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"github_release" => Ok(ChannelType::GithubRelease),
            unrecognized => {
                Err(format!("Unrecognized channel type enum variant: {:?}", unrecognized).into())
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
