use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, put};
use serde::{Deserialize, Serialize};

use crate::auth::AuthenticatedUser;
use crate::db::DbConn;
use crate::geolocation::{calculate_midpoint, MidpointResult};
use crate::models::{Location, NewLocation, NewOrder, Order};
use crate::nominatim::{geocode, reverse_geocode_from_coord, GeocodeResult};
use crate::schema::{locations, orders};

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub product_id: i32,
    pub seller_id: i32,
    pub buyer_location: LocationInput,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LocationInput {
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct LocationUpsertResponse {
    pub id: i32,
    pub user_id: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub id: i32,
    pub product_id: i32,
    pub buyer_id: i32,
    pub seller_id: i32,
    pub status: String,
    pub buyer_location: LocationResponse,
    pub seller_location: LocationResponse,
    pub midpoint_info: MidpointResult,
}

#[derive(Debug, Serialize)]
pub struct LocationResponse {
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct GeocodeRequest {
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct ReverseGeocodeRequest {
    pub latitude: f64,
    pub longitude: f64,
}

#[post("/", data = "<request>")]
pub async fn create_order(
    db: DbConn,
    auth: AuthenticatedUser,
    request: Json<CreateOrderRequest>,
) -> Result<Json<OrderResponse>, Status> {
    let buyer_id = auth.user_id;
    let seller_id = request.seller_id;
    let product_id = request.product_id;
    let buyer_loc_input = request.buyer_location.clone();

    // Create buyer location
    let buyer_location: Location = db
        .run(move |conn| {
            diesel::insert_into(locations::table)
                .values(&NewLocation {
                    user_id: buyer_id,
                    latitude: buyer_loc_input.latitude,
                    longitude: buyer_loc_input.longitude,
                    address: buyer_loc_input.address.clone(),
                })
                .get_result(conn)
        })
        .await
        .map_err(|_| Status::InternalServerError)?;

    // Get or create seller location (default for now)
    let seller_location: Location = db
        .run(move |conn| {
            locations::table
                .filter(locations::user_id.eq(seller_id))
                .first(conn)
                .or_else(|_| {
                    diesel::insert_into(locations::table)
                        .values(&NewLocation {
                            user_id: seller_id,
                            latitude: -6.2088, // Default Jakarta coordinates
                            longitude: 106.8456,
                            address: "Jakarta, Indonesia (Default - seller should update)"
                                .to_string(),
                        })
                        .get_result(conn)
                })
        })
        .await
        .map_err(|_| Status::InternalServerError)?;

    // Create the order
    let buyer_loc_id = buyer_location.id;
    let seller_loc_id = seller_location.id;

    let order: Order = db
        .run(move |conn| {
            diesel::insert_into(orders::table)
                .values(&NewOrder {
                    product_id,
                    buyer_id,
                    seller_id,
                    buyer_location_id: buyer_loc_id,
                    seller_location_id: seller_loc_id,
                })
                .get_result(conn)
        })
        .await
        .map_err(|_| Status::InternalServerError)?;

    // Calculate midpoint
    let midpoint_info = calculate_midpoint(
        buyer_location.latitude,
        buyer_location.longitude,
        seller_location.latitude,
        seller_location.longitude,
    );

    Ok(Json(OrderResponse {
        id: order.id,
        product_id: order.product_id,
        buyer_id: order.buyer_id,
        seller_id: order.seller_id,
        status: order.status,
        buyer_location: LocationResponse {
            latitude: buyer_location.latitude,
            longitude: buyer_location.longitude,
            address: buyer_location.address,
        },
        seller_location: LocationResponse {
            latitude: seller_location.latitude,
            longitude: seller_location.longitude,
            address: seller_location.address,
        },
        midpoint_info,
    }))
}

#[get("/<id>")]
pub async fn get_order(
    db: DbConn,
    auth: AuthenticatedUser,
    id: i32,
) -> Result<Json<OrderResponse>, Status> {
    let user_id = auth.user_id;

    let order: Order = db
        .run(move |conn| orders::table.find(id).first(conn))
        .await
        .map_err(|_| Status::NotFound)?;

    // Verify user is buyer or seller
    if order.buyer_id != user_id && order.seller_id != user_id {
        return Err(Status::Forbidden);
    }

    let buyer_loc_id = order.buyer_location_id;
    let seller_loc_id = order.seller_location_id;

    let (buyer_location, seller_location): (Location, Location) = db
        .run(move |conn| {
            let buyer = locations::table.find(buyer_loc_id).first(conn)?;
            let seller = locations::table.find(seller_loc_id).first(conn)?;
            Ok::<_, diesel::result::Error>((buyer, seller))
        })
        .await
        .map_err(|_| Status::InternalServerError)?;

    let midpoint_info = calculate_midpoint(
        buyer_location.latitude,
        buyer_location.longitude,
        seller_location.latitude,
        seller_location.longitude,
    );

    Ok(Json(OrderResponse {
        id: order.id,
        product_id: order.product_id,
        buyer_id: order.buyer_id,
        seller_id: order.seller_id,
        status: order.status,
        buyer_location: LocationResponse {
            latitude: buyer_location.latitude,
            longitude: buyer_location.longitude,
            address: buyer_location.address,
        },
        seller_location: LocationResponse {
            latitude: seller_location.latitude,
            longitude: seller_location.longitude,
            address: seller_location.address,
        },
        midpoint_info,
    }))
}

#[get("/my-orders")]
pub async fn my_orders(db: DbConn, auth: AuthenticatedUser) -> Result<Json<Vec<Order>>, Status> {
    let user_id = auth.user_id;

    let user_orders: Vec<Order> = db
        .run(move |conn| {
            orders::table
                .filter(
                    orders::buyer_id
                        .eq(user_id)
                        .or(orders::seller_id.eq(user_id)),
                )
                .order(orders::created_at.desc())
                .load(conn)
        })
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(user_orders))
}

#[post("/address", data = "<request>")]
pub async fn geocode_address(request: Json<GeocodeRequest>) -> Result<Json<GeocodeResult>, Status> {
    geocode(&request.address)
        .await
        .map(Json)
        .map_err(|_| Status::NotFound)
}

#[post("/reverse", data = "<request>")]
pub async fn reverse_geocode(
    request: Json<ReverseGeocodeRequest>,
) -> Result<Json<GeocodeResult>, Status> {
    reverse_geocode_from_coord(request.latitude, request.longitude)
        .await
        .map(Json)
        .map_err(|_| Status::NotFound)
}

#[put("/me", data = "<request>")]
pub async fn upsert_my_location(
    db: DbConn,
    auth: AuthenticatedUser,
    request: Json<LocationInput>,
) -> Result<Json<LocationUpsertResponse>, Status> {
    let user_id = auth.user_id;
    let input = request.into_inner();

    let location: Location = db
        .run(move |conn| {
            // Try update first
            let updated = diesel::update(locations::table.filter(locations::user_id.eq(user_id)))
                .set((
                    locations::latitude.eq(input.latitude),
                    locations::longitude.eq(input.longitude),
                    locations::address.eq(input.address.clone()),
                ))
                .execute(conn)?;

            if updated > 0 {
                // Fetch the updated row
                locations::table
                    .filter(locations::user_id.eq(user_id))
                    .first::<Location>(conn)
            } else {
                // Insert if no existing row
                diesel::insert_into(locations::table)
                    .values(&NewLocation {
                        user_id,
                        latitude: input.latitude,
                        longitude: input.longitude,
                        address: input.address,
                    })
                    .get_result::<Location>(conn)
            }
        })
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(LocationUpsertResponse {
        id: location.id,
        user_id: location.user_id,
        latitude: location.latitude,
        longitude: location.longitude,
        address: location.address,
    }))
}
