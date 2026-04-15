#[cfg(test)]
mod user_service {
    use rand::Rng;
    use sqlx::PgPool;

    use crate::{
        models::CreateUserRequest,
        services::{UserService, hash_password},
    };

    #[test]
    fn hash_password_is_not_plaintext() {
        let hash = hash_password("secret123").unwrap();
        assert_ne!(hash, "secret123");
        assert!(!hash.is_empty());
        // argon2 PHC format
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn hash_password_produces_different_hashes() {
        let h1 = hash_password("secret123").unwrap();
        let h2 = hash_password("secret123").unwrap();
        // salts differ per call
        assert_ne!(h1, h2);
    }

    #[sqlx::test]
    async fn list_users_return_stored_users(pool: PgPool) {
        UserService::create_user(&pool, create_random_user_request())
            .await
            .unwrap();
        UserService::create_user(&pool, create_random_user_request())
            .await
            .unwrap();

        let users = UserService::list_users(&pool).await.unwrap();

        assert!(!users.is_empty());
        assert_eq!(users.len(), 2)
    }

    #[sqlx::test]
    async fn get_user_by_id_return_expected_user(pool: PgPool) {
        let req = create_random_user_request();

        let user = UserService::create_user(&pool, req).await.unwrap();

        let result = UserService::get_user_by_id(&pool, user.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(result.id, user.id);
        assert_eq!(result.password_hash, user.password_hash);
    }

    #[sqlx::test]
    async fn create_user_stores_hashed_password(pool: PgPool) {
        let req = CreateUserRequest {
            email: "test@example.com".to_string(),
            password: "secret123".to_string(),
            full_name: Some("Test User".to_string()),
        };

        let user = UserService::create_user(&pool, req).await.unwrap();

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.full_name, "Test User");
        assert_ne!(user.password_hash, "secret123");
        assert!(!user.password_hash.is_empty());
    }

    fn create_random_user_request() -> CreateUserRequest {
        let random_number = rand::rng().random_range(0..1000);
        CreateUserRequest {
            email: format!("test{random_number}@example.com").to_string(),
            password: format!("secret123-{random_number}").to_string(),
            full_name: Some(format!("Test User {random_number}").to_string()),
        }
    }
}
