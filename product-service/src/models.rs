use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = crate::schema::categories)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Category))]
#[diesel(table_name = crate::schema::products)]
pub struct Product {
    pub id: i32,
    pub seller_id: i32,
    pub category_id: i32,
    pub title: String,
    pub description: String,
    pub price: f64,
    pub image_url: Option<String>,
    pub status: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::products)]
pub struct NewProduct {
    pub seller_id: i32,
    pub category_id: i32,
    pub title: String,
    pub description: String,
    pub price: f64,
    pub image_url: Option<String>,
}
