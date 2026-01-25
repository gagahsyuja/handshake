// @generated automatically by Diesel CLI.

diesel::table! {
    email_verifications (id) {
        id -> Int4,
        user_id -> Int4,
        #[max_length = 6]
        code -> Varchar,
        expires_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        email_verified -> Bool,
        created_at -> Timestamp,
    }
}

diesel::joinable!(email_verifications -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(email_verifications, users,);
