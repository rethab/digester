use redis::{Commands, Connection, PipelineCommands, ToRedisArgs};
use serde_cbor;
use serde_derive::{Deserialize, Serialize};
use time::Duration;

use uuid::Uuid;

#[derive(Clone)]
pub struct SessionId(pub Uuid);

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: i32,
    pub username: String,
}

impl ToRedisArgs for SessionId {
    fn write_redis_args(&self, out: &mut Vec<Vec<u8>>) {
        out.push(self.0.as_bytes().to_vec())
    }
}

pub fn session_store(
    conn: &mut Connection,
    id: SessionId,
    data: &SessionData,
    lifetime: Duration,
) -> Result<(), String> {
    let serialized_data = serde_cbor::to_vec(data)
        .map_err(|err| format!("Failed to serialize session data: {:?}", err))?;
    redis::pipe()
        .set(id.clone(), serialized_data)
        .expire(id, lifetime.num_seconds() as usize)
        .query(conn)
        .map_err(|err| format!("Failed to store session in redis: {:?}", err))
}

pub fn session_find(conn: &Connection, id: SessionId) -> Result<Option<SessionData>, String> {
    let maybe_data: Option<Vec<u8>> = conn
        .get(id)
        .map_err(|err| format!("Failed to fetch session from redis: {:?}", err))?;

    match maybe_data {
        None => Ok(None),
        Some(data) => serde_cbor::from_slice(&data[..])
            .map_err(|err| format!("Failed to deserialize redis value: {:?}", err)),
    }
}

pub fn session_delete(conn: &mut Connection, id: SessionId) -> Result<(), String> {
    conn.del(id)
        .map_err(|err| format!("Failed to delete key in redis: {:?}", err))
}

pub fn ratelimit_get(conn: &mut Connection, key: &str) -> Result<usize, String> {
    conn.get(key)
        .map_err(|err| format!("Failed to get key {}: {:?}", key, err))
}

pub fn ratelimit_increment(conn: &mut Connection, key: &str) -> Result<(), String> {
    redis::pipe()
        .incr(key, 1) // redis command incrby
        .expire(key, 59)
        .query(conn)
        .map_err(|err| format!("Failed to increment and get {}: {:?}", key, err))
}

pub fn delete_challenge_store(
    conn: &mut Connection,
    user_id: i32,
    value: &str,
    lifetime: Duration,
) -> Result<(), String> {
    let key = create_delete_challenge_key(user_id);
    redis::pipe()
        .set(key.clone(), value)
        .expire(key, lifetime.num_seconds() as usize)
        .query(conn)
        .map_err(|err| format!("Failed to store delete challenge in redis: {:?}", err))
}

pub fn delete_challenge_get_and_delete(
    conn: &mut Connection,
    user_id: i32,
) -> Result<Option<String>, String> {
    let key = create_delete_challenge_key(user_id);
    let result = conn
        .get(key.clone())
        .map_err(|err| format!("Failed to get key {}: {:?}", key, err));

    // delete the key regardless of whether retrieving it worked. in any
    // case if the user wants to try again, we need to generate a fresh challenge.
    conn.del(key.clone())
        .map_err(|err| format!("Failed to delete key {}: {:?}", key, err))?;
    result
}

fn create_delete_challenge_key(user_id: i32) -> String {
    format!("delete_challenge.{}", user_id)
}
