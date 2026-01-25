// @generated automatically by Diesel CLI.

diesel::table! {
    categories (id) {
        id -> Int4,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 100]
        slug -> Varchar,
        #[max_length = 255]
        icon -> Nullable<Varchar>,
    }
}

diesel::table! {
    products (id) {
        id -> Int4,
        seller_id -> Int4,
        category_id -> Int4,
        #[max_length = 255]
        title -> Varchar,
        description -> Text,
        price -> Float8,
        #[max_length = 500]
        image_url -> Nullable<Varchar>,
        #[max_length = 50]
        status -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::joinable!(products -> categories (category_id));

diesel::allow_tables_to_appear_in_same_query!(categories, products,);
