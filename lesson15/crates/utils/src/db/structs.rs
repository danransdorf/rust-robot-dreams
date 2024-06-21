use paste::paste;
use serde::{Deserialize, Serialize};

use crate::db::schema::{messages, users};
use diesel::prelude::*;

macro_rules! diesel_struct {
    ($name:ident, $table:ident, $($field:ident: $type:ty),*) => {
        #[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
        #[diesel(table_name = $table)]
        pub struct $name {
            pub id: Option<i32>,
            $(pub $field: $type),*
        }


        paste!{
            #[derive(Insertable, Serialize, Deserialize, Debug, Clone)]
            #[diesel(table_name = $table)]
            pub struct [<ToBeInserted $name>] {
                $(pub $field: $type),*
            }

            impl [<ToBeInserted $name>] {
                pub fn new($($field: $type),*) -> Self {
                    [<ToBeInserted $name>] {$($field),*}
                }
            }
        }
    };
}

/*
Generate structs representing the data objects to be inserted (without id) ToBeInserted{name}.
And generate structs representing the response from the database (with auto-incremented id)
*/
diesel_struct!(
    User,
    users,
    username: String,
    password: String,
    salt: Vec<u8>
);
diesel_struct!(Message, messages, user_id: i32, content: Vec<u8>);
