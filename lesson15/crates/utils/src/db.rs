use crate::errors::DBError;
use crate::{serialize_data, MessageData};
use bcrypt::{hash_with_salt, verify, DEFAULT_COST};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use rand::RngCore;

pub mod schema;
use schema::messages as messages_schema;
pub mod structs;
use structs::{Message, ToBeInsertedMessage, ToBeInsertedUser, User};

static DB_PATH: &'static str = "chat.db";

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
        use schema::users::dsl::{id as id_field, users as users_table};

        let mut salt = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt);

        let hashed_password = hash_with_salt(password, DEFAULT_COST, salt).unwrap();
        let new_user =
            ToBeInsertedUser::new(username.clone(), hashed_password.to_string(), salt.to_vec());

        let mut conn = self.pool.get().map_err(|_| DBError::ConnectionError)?;
        diesel::insert_into(users_table)
            .values(&new_user)
            .execute(&mut conn)
            .map_err(|e| {
                println!("{}", e);
                return DBError::UserInsertionError;
            })?;

        // Diesel doesn't support RETURNING clause for SQLite, so I have to fetch the last user manually
        let user = users_table
            .order(id_field.desc())
            .first(&mut conn)
            .map_err(|_| DBError::UserNotFoundError)?;

        println!("User created");

        Ok(user)
    }
    pub fn get_user_id(&self, username: &str) -> Result<i32> {
        use schema::users::dsl::{username as username_field, users as users_table};

        let mut conn = self.pool.get().map_err(|_| DBError::ConnectionError)?;
        let user: User = users_table
            .filter(username_field.eq(username))
            .first(&mut conn)
            .map_err(|_| DBError::UserNotFoundError)?;

        Ok(user.id.unwrap())
    }

    pub fn check_password(&self, username: &str, password: &str) -> Result<bool, DBError> {
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

    pub fn save_message(&self, user_id: i32, message: MessageData) -> Result<Message> {
        use schema::messages::dsl::{id as id_field, messages as messages_table};

        let serialized_message = serialize_data(message)?;
        let new_message = ToBeInsertedMessage::new(user_id, serialized_message);

        let mut conn = self.pool.get().map_err(|_| DBError::ConnectionError)?;
        diesel::insert_into(messages_schema::table)
            .values(&new_message)
            .execute(&mut conn)
            .map_err(|_| DBError::MessageInsertionError)?;

        // Diesel doesn't support RETURNING clause for SQLite, so I have to fetch the last message manually
        let message = messages_table
            .order(id_field.desc())
            .first(&mut conn)
            .map_err(|_| DBError::UserNotFoundError)?;

        Ok(message)
    }

    pub fn get_user(&self, user_id: i32) -> Result<User, DBError> {
        use schema::users::dsl::{id as id_field, users as users_table};

        let mut conn = self.pool.get().map_err(|_| DBError::ConnectionError)?;
        let user = users_table
            .filter(id_field.eq(user_id))
            .first(&mut conn)
            .map_err(|_| DBError::UserNotFoundError)?;

        Ok(user)
    }

    pub fn read_history(&self, amount: i32) -> Result<Vec<Message>> {
        use schema::messages::dsl::{id as id_field, messages as messages_table};

        let mut conn = self.pool.get().map_err(|_| DBError::ConnectionError)?;
        let messages: Vec<Message> = messages_table
            .order(id_field.desc())
            .limit(amount as i64)
            .order(id_field.asc())
            .load(&mut conn)
            .map_err(|_| DBError::MessageHistoryError)?;

        Ok(messages)
    }
}
