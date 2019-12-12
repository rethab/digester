use redis::{Commands, Connection, ToRedisArgs};
use serde_cbor;
use serde_derive::{Deserialize, Serialize};

use uuid::Uuid;

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
) -> Result<(), String> {
    let serialized_data = serde_cbor::to_vec(data)
        .map_err(|err| format!("Failed to serialize session data: {:?}", err))?;
    conn.set(id, serialized_data)
        .map_err(|err| format!("Failed to store session in redis: {:?}", err))
}
