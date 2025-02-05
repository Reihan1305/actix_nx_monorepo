use sqlx::{query, query_as, PgPool};
use uuid::Uuid;

use crate::utils::types_utils::AccessToken;

use super::user_models::{LoginQueryPayload, RegisterData, RegisterPayload};


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

    pub async fn create_refresh_token(
        token:&str,
        userid:Uuid,
        db_pool: &PgPool
    )->Result<(),String>{
        let _ = query!(
            r#"
            INSERT INTO "refresh_token" 
            (userid,refreshtoken) VALUES
            ($1,$2)
            "#,
            userid,
            token
        ).execute(db_pool).await.map_err(|error|{
            format!("error database: {}",error)
        });
        Ok(())
    }
    
    pub async fn find_refresh_token(
        token:String,
        userid:Uuid,
        db_pool: &PgPool
    )-> Result<Uuid,String>{
        match query!(
            r#"
                SELECT * FROM "refresh_token" where userid = $1 AND refreshtoken = $2
            "#,
            userid,
            token
        ).fetch_one(db_pool).await{
            Ok(_)=>Ok(userid),
            Err(error)=>Err(format!("error database: {}", error))
        }

    }

    pub async fn login_query(
        email: Option<String>,
        username: Option<String>,
        db_pool: &PgPool
    )->Result<LoginQueryPayload,String>{
        
        if email.is_none() && username.is_none(){
            return Err(String::from("email and username is empty!"));
        }
        
        let login_payload = if let Some(email) = email {
            query_as!(
                LoginQueryPayload,
                r#"SELECT id, email, username, password FROM "user" WHERE email = $1"#,
                email
            )
            .fetch_one(db_pool)
            .await
        } else {
            query_as!(
                LoginQueryPayload,
                r#"SELECT id, email, username, password FROM "user" WHERE username = $1"#,
                username
            )
            .fetch_one(db_pool)
            .await
        };
        match login_payload {
            Ok(payload)=> Ok(payload),
            Err(message)=>Err(format!("{}",message))
        }
    }

    pub async fn find_user_by_id(
        id: Uuid,
        db_pool: &PgPool
    )-> Result<AccessToken,String>{
        match query_as!(
            AccessToken,
            r#"
            SELECT id, username, email FROM "user" where id = $1;
            "#,
            id
        ).fetch_one(db_pool).await{
            Ok(user)=>Ok(user),
            Err(error)=> Err(format!("db error: {}", error))
        }
    }
}