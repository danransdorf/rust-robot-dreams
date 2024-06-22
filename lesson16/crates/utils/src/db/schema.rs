// @generated automatically by Diesel CLI.

diesel::table! {
    messages (id) {
        id -> Nullable<Integer>,
        user_id -> Integer,
        content -> Binary,
    }
}

diesel::table! {
    users (id) {
        id -> Nullable<Integer>,
        username -> Text,
        password -> Text,
        salt -> Binary,
    }
}

diesel::allow_tables_to_appear_in_same_query!(messages, users,);
