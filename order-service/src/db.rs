use rocket_sync_db_pools::{database, diesel};

#[database("order_db")]
pub struct DbConn(diesel::PgConnection);
