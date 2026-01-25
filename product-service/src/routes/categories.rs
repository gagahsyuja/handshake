use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::get;
use diesel::prelude::*;

use crate::db::DbConn;
use crate::models::Category;
use crate::schema::categories;
use crate::routes::products::ProductResponse;

#[get("/")]
pub async fn list_categories(db: DbConn) -> Result<Json<Vec<Category>>, Status> {
    let categories: Vec<Category> = db.run(|conn| {
        categories::table
            .order(categories::name.asc())
            .load(conn)
    }).await.map_err(|_| Status::InternalServerError)?;

    Ok(Json(categories))
}

#[get("/<slug>/products?<limit>")]
pub async fn get_category_products(
    db: DbConn,
    slug: String,
    limit: Option<i64>,
) -> Result<Json<Vec<ProductResponse>>, Status> {
    use crate::schema::products;
    use crate::models::Product;

    let limit = limit.unwrap_or(20).min(100);

    let results: Vec<(Product, Category)> = db.run(move |conn| {
        products::table
            .inner_join(categories::table.on(products::category_id.eq(categories::id)))
            .filter(categories::slug.eq(&slug))
            .filter(products::status.eq("active"))
            .order(products::created_at.desc())
            .limit(limit)
            .load(conn)
    }).await.map_err(|_| Status::InternalServerError)?;

    let response: Vec<ProductResponse> = results.into_iter().map(|(p, c)| {
        ProductResponse {
            id: p.id,
            seller_id: p.seller_id,
            category_id: p.category_id,
            category_name: c.name,
            title: p.title,
            description: p.description,
            price: p.price,
            image_url: p.image_url,
            status: p.status,
        }
    }).collect();

    Ok(Json(response))
}
