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
