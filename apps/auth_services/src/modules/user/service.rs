
use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use lapin::{options::BasicPublishOptions, publisher_confirm::PublisherConfirm, types::FieldTable, BasicProperties};
use log::info;
use logger_libs::{error_logger, info_logger, warning_logger};
use pgsql_libs::DbPool;
use r2d2_redis::redis::{Commands, RedisError};
use rabbitmq_libs::RabbitMqPool;
use redis_libs::RedisPool;
use serde_json::json;

use jwt_libs::{{decode_refresh_token, generate_access_token, generate_refresh_token},types::{AccessToken, RefreshToken, TokenClaims}};

use super::{model::{LoginData, LoginPayload, RegisterData, RegisterPayload}, query::UserQuery};



pub struct UserServices{}

impl UserServices {
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
                                        let _ = Self::delete_access_token(redis_pool);
                                        let _ = Self::store_access_token(token.clone(), redis_pool);
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
        token: String,
        redis_pool: &RedisPool,
    ) -> Result<(), String> {
        let redis_key = format!("access_token");

        let mut conn = redis_pool.get().map_err(|e| e.to_string())?;

        let set_data: Result<String, RedisError> = conn.set(&redis_key, token.clone());

        let ttl = 12000;
        let _ = conn.expire::<String,String>(redis_key, ttl);
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

    pub fn delete_access_token(
        redis_pool: &RedisPool
    )-> Result<(),String>{
        let redis_key = format!("access_token");

        let mut conn = redis_pool.get().map_err(|e| e.to_string())?;

        let set_data: Result<String, RedisError> = conn.del(&redis_key);
        match set_data {
            Ok(data)=>{
                info!("data inserted: {}",data);
                Ok(())
            },
            Err(error)=>{
                Err(format!("error redis: {}", error))
            }
        }    }

    pub async fn access_token(
        token: AccessToken,
        db_pool: &DbPool
    )-> Result<AccessToken,String>{
        match UserQuery::find_user_by_id(token.id, db_pool).await{
            Ok(user)=>{
                if user.email != token.email{
                    return Err("invalid token".to_string());
                }else{
                    Ok(user)
                }
            },
            Err(error)=>{
                return Err(error)
            }
        }
    }

    //register queue
    pub async fn register(
        log_id: &str,
        mut data: RegisterData,
        db_pool: &DbPool,
        rabbit_pool: &RabbitMqPool,
    ) -> Result<RegisterPayload, String> {
        let argon2 = Argon2::default();
        let salt: SaltString = SaltString::generate(&mut OsRng);
        let password_hash = match argon2.hash_password(data.password.as_bytes(), &salt) {
            Ok(hash) => {
                info_logger(log_id, "register", "hash_password");
                hash.to_string()
            },
            Err(e) => {
                warning_logger(log_id, "register", "Hash_password", &format!("{}",e));
                return Err(format!("Password hash error: {}", e))
            }
        };

        info_logger(log_id, "register", "hash_password");
        data.password = password_hash;

        let conn = match rabbit_pool.get().await {
            Ok(conn) => {
                info_logger(log_id, "register","Redis_connect");
                conn
            },
            Err(err) => {
                error_logger(log_id, "register","Redis_connect",&format!("{}",err));
                return Err(format!("RabbitMQ connection error: {}", err));
            }
        };

        let channel = match conn.create_channel().await {
            Ok(channel) => {
                info_logger(log_id, "register", "Rabbit_connect");
                channel
            },
            Err(err) => {
                error_logger(log_id, "register", "Rabbit_connect", &format!("{}",err));
                return Err(format!("RabbitMQ channel error: {}", err));
            }
        };

        if let Err(err) = channel
            .queue_declare(
                "register_queue",
                lapin::options::QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
        {
            error_logger(log_id, "register",  "Rabbit_queue_declare", &format!("{}",err));
            return Err(format!("RabbitMQ queue declare error: {}", err));
        }

        let payload = json!({
            "email": data.email,
            "username": data.username,
            "password": data.password,
        });

        let payload_bytes = payload.to_string().into_bytes();

        let _: PublisherConfirm = match channel
            .basic_publish(
                "",
                "register_queue",
                BasicPublishOptions::default(),
                &payload_bytes,
                BasicProperties::default(),
            )
            .await
        {
            Ok(confirm) => {
                info_logger(log_id, "register","Rabbit_publish_confirm");
                confirm
            },
            Err(err) => {
                error_logger(log_id, "register", "Rabbit_publish_confirm", &format!("{}",err));
                return Err(format!("RabbitMQ publish error: {}", err));
            }
        };

        info!("✅ Successfully published register request to queue");

        match UserQuery::create_user(data, db_pool).await {
            Ok(register_payload) => {
                info_logger(log_id, "register", "Create_user");

                Ok(register_payload)
            },
            Err(error) => {
                error_logger(log_id, "register", "Create_user", &format!("{}",error));
                Err(format!("Database error: {}", error))
        },
        }
    }
}