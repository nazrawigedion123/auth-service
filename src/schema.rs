// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 100]
        username -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        password_hash -> Nullable<Text>,
        #[max_length = 100]
        display_name -> Varchar,
        #[max_length = 100]
        user_role -> Varchar,
        is_active -> Bool,
        last_login -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
