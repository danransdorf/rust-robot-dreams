use diesel::prelude::*;

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

macro_rules! diesel_struct {
    ($name:ident, $table:ident, $($field:ident: $type:ty),*) => {
        #[derive(Queryable, Insertable)]
        #[diesel(table_name = $table)]
        pub struct $name {
            pub id: i32,
            $(pub $field: $type),*
        }

        impl $name {
            pub fn new($($field: $type),*) -> Self {
                $name {id:0, $($field),*}
            }
        }
    };
}

diesel_struct!(
    User,
    users,
    username: String,
    password: String,
    salt: Vec<u8>
);
diesel_struct!(Message, messages, user_id: i32, content: Vec<u8>);
