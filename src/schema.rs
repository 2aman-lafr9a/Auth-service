// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Integer,
        #[max_length = 50]
        username -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        #[max_length = 10]
        role -> Varchar,
    }
}
