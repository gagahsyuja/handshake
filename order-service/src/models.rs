use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = crate::schema::locations)]
pub struct Location {
    pub id: i32,
    pub user_id: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::locations)]
pub struct NewLocation {
    pub user_id: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = crate::schema::orders)]
pub struct Order {
    pub id: i32,
    pub product_id: i32,
    pub buyer_id: i32,
    pub seller_id: i32,
    pub buyer_location_id: i32,
    pub seller_location_id: i32,
    pub status: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::orders)]
pub struct NewOrder {
    pub product_id: i32,
    pub buyer_id: i32,
    pub seller_id: i32,
    pub buyer_location_id: i32,
    pub seller_location_id: i32,
}
