diesel::table! {
    users {
        id -> Integer,
        username -> Text,
        password -> Text,
        salt -> Binary,
    }
}

diesel::table! {
    messages {
        id -> Integer,
        user_id -> Integer,
        content -> Binary,
    }
}
