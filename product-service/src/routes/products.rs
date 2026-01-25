use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::{get, post, put, delete};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;

use crate::db::DbConn;
use crate::models::{Product, NewProduct, Category};
use crate::schema::{products, categories};
use crate::auth::AuthenticatedUser;

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub id: i32,
    pub seller_id: i32,
    pub category_id: i32,
    pub category_name: String,
    pub title: String,
    pub description: String,
    pub price: f64,
    pub image_url: Option<String>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub category_id: i32,
    pub title: String,
    pub description: String,
    pub price: f64,
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub image_url: Option<String>,
    pub status: Option<String>,
}

#[get("/?<category_id>&<limit>&<offset>")]
pub async fn list_products(
    db: DbConn,
    category_id: Option<i32>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Json<Vec<ProductResponse>>, Status> {
    let limit = limit.unwrap_or(20).min(100);
    let offset = offset.unwrap_or(0);

    let products: Vec<(Product, Category)> = db.run(move |conn| {
        let mut query = products::table
            .inner_join(categories::table.on(products::category_id.eq(categories::id)))
            .into_boxed();

        if let Some(cat_id) = category_id {
            query = query.filter(products::category_id.eq(cat_id));
        }

        query
            .filter(products::status.eq("active"))
            .order(products::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load(conn)
    }).await.map_err(|_| Status::InternalServerError)?;

    let response: Vec<ProductResponse> = products.into_iter().map(|(p, c)| {
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

#[get("/<id>")]
pub async fn get_product(
    db: DbConn,
    id: i32,
) -> Result<Json<ProductResponse>, Status> {
    let product: (Product, Category) = db.run(move |conn| {
        products::table
            .inner_join(categories::table.on(products::category_id.eq(categories::id)))
            .filter(products::id.eq(id))
            .first(conn)
    }).await.map_err(|_| Status::NotFound)?;

    Ok(Json(ProductResponse {
        id: product.0.id,
        seller_id: product.0.seller_id,
        category_id: product.0.category_id,
        category_name: product.1.name,
        title: product.0.title,
        description: product.0.description,
        price: product.0.price,
        image_url: product.0.image_url,
        status: product.0.status,
    }))
}

#[post("/", data = "<request>")]
pub async fn create_product(
    db: DbConn,
    auth: AuthenticatedUser,
    request: Json<CreateProductRequest>,
) -> Result<Json<Product>, Status> {
    let new_product = NewProduct {
        seller_id: auth.user_id,
        category_id: request.category_id,
        title: request.title.clone(),
        description: request.description.clone(),
        price: request.price,
        image_url: request.image_url.clone(),
    };

    let product: Product = db.run(move |conn| {
        diesel::insert_into(products::table)
            .values(&new_product)
            .get_result(conn)
    }).await.map_err(|_| Status::InternalServerError)?;

    Ok(Json(product))
}

#[put("/<id>", data = "<request>")]
pub async fn update_product(
    db: DbConn,
    auth: AuthenticatedUser,
    id: i32,
    request: Json<UpdateProductRequest>,
) -> Result<Json<Product>, Status> {
    let user_id = auth.user_id;
    
    // Check ownership
    let product: Product = db.run(move |conn| {
        products::table.find(id).first(conn)
    }).await.map_err(|_| Status::NotFound)?;

    if product.seller_id != user_id {
        return Err(Status::Forbidden);
    }

    let updated: Product = db.run(move |conn| {
        let target = products::table.find(id);
        
        if let Some(ref title) = request.title {
            diesel::update(target)
                .set(products::title.eq(title))
                .execute(conn)?;
        }
        if let Some(ref description) = request.description {
            diesel::update(target)
                .set(products::description.eq(description))
                .execute(conn)?;
        }
        if let Some(price) = request.price {
            diesel::update(target)
                .set(products::price.eq(price))
                .execute(conn)?;
        }
        if let Some(ref status) = request.status {
            diesel::update(target)
                .set(products::status.eq(status))
                .execute(conn)?;
        }
        
        products::table.find(id).first(conn)
    }).await.map_err(|_| Status::InternalServerError)?;

    Ok(Json(updated))
}

#[delete("/<id>")]
pub async fn delete_product(
    db: DbConn,
    auth: AuthenticatedUser,
    id: i32,
) -> Result<Status, Status> {
    let user_id = auth.user_id;
    
    // Check ownership
    let product: Product = db.run(move |conn| {
        products::table.find(id).first(conn)
    }).await.map_err(|_| Status::NotFound)?;

    if product.seller_id != user_id {
        return Err(Status::Forbidden);
    }

    db.run(move |conn| {
        diesel::delete(products::table.find(id)).execute(conn)
    }).await.map_err(|_| Status::InternalServerError)?;

    Ok(Status::NoContent)
}
