//! Business logic and services
//!
//! This module contains the core business logic for financial operations,
//! calculations, and data processing.

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use sqlx::PgPool;

use crate::models::{CreateUserRequest, User};

pub(crate) fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("password hashing failed: {e}"))?
        .to_string();
    Ok(hash)
}

pub struct UserService;

impl UserService {
    pub async fn create_user(pool: &PgPool, req: CreateUserRequest) -> anyhow::Result<User> {
        let hash = hash_password(&req.password)?;
        let full_name = req.full_name.unwrap_or_default();

        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (email, password_hash, full_name) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(req.email)
        .bind(hash)
        .bind(full_name)
        .fetch_one(pool)
        .await?;
        Ok(user)
    }

    pub async fn get_user_by_id(pool: &PgPool, id: i64) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(user)
    }

    pub async fn list_users(pool: &PgPool) -> anyhow::Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER by id")
            .fetch_all(pool)
            .await?;
        Ok(users)
    }
}
