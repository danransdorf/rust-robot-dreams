use paste::paste;
use serde::{Deserialize, Serialize};

use crate::db::schema::{messages, users};
use diesel::prelude::*;

/// Generate structs representing the data objects to be inserted (without id) ToBeInserted{name}.
/// And generate structs representing the response from the database (with auto-incremented id)
///
/// # Arguments
/// * `$name` - The name of the struct
/// * `$table` - The name of the table in the database
/// * ...`$field: $type` - The fields of the struct
///
/// # Example
///
/// ```
/// diesel_struct!(
///     User,
///     users,
///     username: String,
///     password: String,
///     salt: Vec<u8>
/// );
/// ```
/// generates
/// ```
/// #[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
/// #[diesel(table_name = users)]
/// pub struct User {
///     pub id: Option<i32>,
///     pub username: String,
///     pub password: String,
///     pub salt: Vec<u8>
/// }
///
/// #[derive(Insertable, Serialize, Deserialize, Debug, Clone)]
/// #[diesel(table_name = users)]
/// pub struct ToBeInsertedUser {
///     pub username: String,
///     pub password: String,
///     pub salt: Vec<u8>
/// }
///
/// impl ToBeInsertedUser {
///     pub fn new(username: String, password: String, salt: Vec<u8>) -> Self {
///         ToBeInsertedUser { username, password, salt }
///     }
/// }
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

diesel_struct!(
    User,
    users,
    username: String,
    password: String,
    salt: Vec<u8>
);
diesel_struct!(Message, messages, user_id: i32, content: Vec<u8>);
