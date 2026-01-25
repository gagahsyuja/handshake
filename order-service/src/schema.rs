// @generated automatically by Diesel CLI.

diesel::table! {
    locations (id) {
        id -> Int4,
        user_id -> Int4,
        latitude -> Float8,
        longitude -> Float8,
        address -> Text,
    }
}

diesel::table! {
    orders (id) {
        id -> Int4,
        product_id -> Int4,
        buyer_id -> Int4,
        seller_id -> Int4,
        buyer_location_id -> Int4,
        seller_location_id -> Int4,
        #[max_length = 50]
        status -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(locations, orders,);
