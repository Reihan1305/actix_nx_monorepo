use sqlx::{query_as, PgPool};

use super::user_models::{RegisterData, RegisterPayload};


pub struct UserQuery {}

impl UserQuery {
    pub async fn create_user(
        data: RegisterData,
        db_pool: &PgPool
    ) -> Result<RegisterPayload, String> {
        let existing_user = query_as!(
            RegisterPayload,
            r#"
            SELECT id, email, username FROM "user" 
            WHERE email = $1 OR username = $2
            
            "#,
            data.email,
            data.username
        )
        .fetch_optional(db_pool)
        .await
        .map_err(|err| format!("Database error: {}", err))?;

        if existing_user.is_some() {
            return Err("Email atau username sudah digunakan".to_string());
        }

        query_as!(
            RegisterPayload,
            r#"
            INSERT INTO "user" 
            (email, username, password)
            VALUES
            ($1, $2, $3)
            RETURNING
            id, email, username
            "#,
            data.email,
            data.username,
            data.password
        )
        .fetch_one(db_pool)
        .await
        .map_err(|err| format!("error query: {}", err)) 
    }
}