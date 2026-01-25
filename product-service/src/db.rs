use rocket_sync_db_pools::{database, diesel};

#[database("product_db")]
pub struct DbConn(diesel::PgConnection);
