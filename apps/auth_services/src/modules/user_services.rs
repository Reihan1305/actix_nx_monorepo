use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use log::info;
use pgsql_libs::DbPool;
use r2d2_redis::redis::{Commands, RedisError};
use redis_libs::RedisPool;
use uuid::Uuid;

use crate::utils::{jwt_utils::{decode_refresh_token, generate_access_token, generate_refresh_token}, types_utils::{AccessToken, RefreshToken, TokenClaims}};

use super::{user_models::{LoginData, LoginPayload, RegisterData, RegisterPayload}, user_query::UserQuery};



pub struct UserServices{}

impl UserServices {
    pub async fn register(
         mut data:RegisterData,
        db_pool:&DbPool
    )->Result<RegisterPayload,String>{
        
        let argon2 = Argon2::default();
        let salt :SaltString = SaltString::generate(&mut OsRng);

        let password_hash = match argon2.hash_password(data.password.as_bytes(), &salt) {
            Ok(hash) => hash.to_string(),
            Err(e) => {
                return Err(format!("{}",e))
            }
        };

        data.password = password_hash;
        match UserQuery::create_user(data, &db_pool).await {
            Ok(register_payload)=>Ok(register_payload),
            Err(error)=>Err(error)
        }
    }

    pub async fn login(
        data: LoginData,
        db_pool: &DbPool
    ) -> Result<LoginPayload, String> {
        // Cek data login berdasarkan email dan username
        let login_data = match UserQuery::login_query(data.email, data.username, db_pool).await {
            Ok(login_data) => login_data,
            Err(error) => return Err(format!("Database error: {}", error)),
        };
    
        let argon2 = Argon2::default();
    
        // Parse hash password yang disimpan di database
        let parsed_hash = match PasswordHash::new(&login_data.password) {
            Ok(parsed_hash) => parsed_hash,
            Err(_) => return Err("Error parsing stored password hash".to_string()),
        };
    
        // Verifikasi password yang dimasukkan dengan yang ada di database
        if let Err(err) = argon2.verify_password(data.password.as_bytes(), &parsed_hash) {
            return Err(format!("Invalid password: {}", err));
        }
    
        // Membuat data refresh token
        let refresh_token_data = RefreshToken {
            id: login_data.id,
        };
    
        // Generate refresh token
        match generate_refresh_token(refresh_token_data) {
            Ok(refresh_token) => {
                // Simpan refresh token di database
                match UserQuery::create_refresh_token(&refresh_token, login_data.id, db_pool).await {
                    Ok(_) => {
                        // Generate access token
                        let access_token_data = AccessToken {
                            id: login_data.id,
                            email: login_data.email.clone(),
                            username: login_data.username.clone(),
                        };
    
                        match generate_access_token(access_token_data) {
                            Ok(access_token) => {
                                // Kembalikan LoginPayload yang berisi access token dan refresh token
                                let payload = LoginPayload {
                                    id: login_data.id,
                                    email: login_data.email,
                                    username: login_data.username,
                                    access_token,
                                    refresh_token,
                                };
    
                                Ok(payload)
                            }
                            Err(error) => Err(format!("Error generating access token: {}", error)),
                        }
                    }
                    Err(error) => Err(format!("Error saving refresh token to database: {}", error)),
                }
            }
            Err(error) => Err(format!("Error generating refresh token: {}", error)),
        }
    }    

    pub async fn refresh_token(
        token:String,
        db_pool: &DbPool,
        redis_pool: &RedisPool
    )->Result<String,String>{
        let refresh_token: Result<jsonwebtoken::TokenData<TokenClaims<RefreshToken>>, String> = decode_refresh_token(&token);

        match refresh_token{
            Ok(decode_token)=>{
                match UserQuery::find_refresh_token(token, decode_token.claims.token.id, db_pool).await{
                    Ok(user_id)=>{
                        match UserQuery::find_user_by_id(user_id, db_pool).await {
                            Ok(user)=>{
                                let access_token = generate_access_token(user);
                                match access_token {
                                    Ok(token)=>{
                                        let _ = Self::store_access_token(user_id, token.clone(), redis_pool);
                                        Ok(token)
                                    },
                                    Err(error)=> Err(format!("generate token error: {}",error))
                                }
                            },
                            Err(error)=>{
                                Err(format!("user not found :{}",error))
                            }
                        }
                    },
                    Err(error)=>Err(error)
                }
            },
            Err(error)=> return Err(error)
        }
    }

    pub fn store_access_token(
        user_id: Uuid,
        token: String,
        redis_pool: &RedisPool,
    ) -> Result<(), String> {
        let redis_key = format!("access_token:{}", user_id);

        let mut conn = redis_pool.get().map_err(|e| e.to_string())?;

        let set_data: Result<String, RedisError> = conn.set(&redis_key, token.clone());
        match set_data {
            Ok(data)=>{
                info!("data inserted: {}",data);
                Ok(())
            },
            Err(error)=>{
                Err(format!("error redis: {}", error))
            }
        }
    }

}