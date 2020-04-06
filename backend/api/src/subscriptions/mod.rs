use diesel::pg::PgConnection;
use lib_db as db;

pub mod search;

pub fn delete(db: &PgConnection, id: i32) -> Result<(), String> {
    db.build_transaction()
        .run(|| {
            db::digests_delete_by_subscription_id(db, id)?;
            db::subscriptions_delete_by_id(db, id)
        })
        .map_err(|err| format!("Failed to delete subscriptions and user {}: {:?}", id, err,))
}
