use bcrypt::{hash_with_salt, verify, DEFAULT_COST};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::sqlite::SqliteConnection;
use rand::RngCore;
use utils::{serialize_data, MessageData};

mod schema;
use schema::{messages as messages_schema, users as users_schema};
mod errors;
use errors::DBError;

static DB_PATH: &'static str = "chat.db";

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

diesel_struct!(User, users_schema, username: String, password: String, salt: Vec<u8>);
diesel_struct!(Message, messages_schema, user_id: i32, content: Vec<u8>);

type SqlitePool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub struct DB {
    pool: SqlitePool,
}

use anyhow::Result;

impl DB {
    pub fn new() -> Result<Self> {
        println!("Creating/reading database chat.db");
        let manager = ConnectionManager::<SqliteConnection>::new(DB_PATH);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .map_err(|_| DBError::PoolCreationError)?;
        Ok(Self { pool })
    }

    pub fn create_user(&self, username: String, password: String) -> Result<User> {
        let mut salt = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt);

        let hashed_password = hash_with_salt(password, DEFAULT_COST, salt).unwrap();
        let new_user = User::new(username, hashed_password.to_string(), salt.to_vec());

        let mut conn = self.pool.get().map_err(|_| DBError::ConnectionError)?;
        diesel::insert_into(users_schema::table)
            .values(&new_user)
            .execute(&mut conn)
            .map_err(|_| DBError::UserInsertionError)?;

        Ok(new_user)
    }

    pub fn check_password(&self, username: &str, password: &str) -> Result<bool> {
        use schema::users::dsl::{username as username_field, users as users_table};

        let mut conn = self.pool.get().map_err(|_| DBError::ConnectionError)?;
        let user: User = users_table
            .filter(username_field.eq(username))
            .first(&mut conn)
            .map_err(|_| DBError::UserNotFoundError)?;

        let verified =
            verify(password, &user.password).map_err(|_| DBError::PasswordVerificationError)?;

        Ok(verified)
    }

    pub fn save_message(&self, user_id: i32, message: MessageData) -> Result<()> {
        let serialized_message = serialize_data(message)?;
        let new_message: Message = Message::new(user_id, serialized_message);

        let mut conn = self.pool.get().map_err(|_| DBError::ConnectionError)?;
        diesel::insert_into(messages_schema::table)
            .values(new_message)
            .execute(&mut conn)
            .map_err(|_| DBError::MessageInsertionError)?;

        Ok(())
    }

    pub fn read_message(&self, message_id: i32) -> Result<Message> {
        use schema::messages::dsl::{id as id_field, messages as messages_table};

        let mut conn = self.pool.get().map_err(|_| DBError::ConnectionError)?;
        let message: Message = messages_table
            .filter(id_field.eq(message_id))
            .first(&mut conn)
            .map_err(|_| DBError::MessageNotFoundError)?;

        Ok(message)
    }

    pub fn read_history(&self, amount: i32) -> Result<Vec<Message>> {
        use schema::messages::dsl::{id as id_field, messages as messages_table};

        let mut conn = self.pool.get().map_err(|_| DBError::ConnectionError)?;
        let messages: Vec<Message> = messages_table
            .order(id_field.desc())
            .limit(amount as i64)
            .load(&mut conn)
            .map_err(|_| DBError::MessageHistoryError)?;

        Ok(messages)
    }
}
