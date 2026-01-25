use rocket_sync_db_pools::{database, diesel};

#[database("auth_db")]
pub struct DbConn(diesel::PgConnection);
