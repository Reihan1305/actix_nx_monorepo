use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use lapin::{options::BasicPublishOptions, publisher_confirm::PublisherConfirm, types::FieldTable, BasicProperties};
use log::info;
use logger_libs::Logger;
use pgsql_libs::DbPool;
use r2d2_redis::redis::{Commands, RedisError};
use rabbitmq_libs::RabbitMqPool;
use redis_libs::{create_redis_connection, RedisPool};
use serde_json::json;

use jwt_libs::{{decode_refresh_token, generate_access_token, generate_refresh_token},types::{AccessToken, RefreshToken}};

use super::{model::{LoginData, LoginPayload, RegisterData, RegisterPayload}, query::UserQuery};

pub struct UserServices{}

impl UserServices {
    pub async fn login(
        log_id: &str,
        data: LoginData,
        db_pool: &DbPool,
        redis_pool: &RedisPool
    ) -> Result<LoginPayload, String> {
        let handler_name = "login_service";
        let login_data: super::model::LoginQueryPayload = match UserQuery::login_query(data.email.clone(), data.username.clone(), db_pool).await {
            Ok(login_data) => {
                Logger::debug_logger(handler_name, log_id, &data, "login_services.data_validate", &login_data);
                login_data
            },
            Err(error) => {
                Logger::warning_logger(handler_name, log_id, "login_services.data_validate", &error);
                return Err(format!("Database error: {}", error))
            }
        };
    
        let argon2 = Argon2::default();
    
        let parsed_hash = match PasswordHash::new(&login_data.password) {
            Ok(parsed_hash) => {
                Logger::info_logger(&handler_name, log_id, "login_service.password_validate");
                parsed_hash
            },
            Err(err) => {
                Logger::warning_logger(&handler_name, log_id, "login_service.password_validate", &format!("{}",err));
                return Err("Error parsing stored password hash".to_string())
            },
        };
    
        if let Err(err) = argon2.verify_password(data.password.as_bytes(), &parsed_hash) {
            Logger::warning_logger(&handler_name, log_id, "login_service.password_validate", &format!("{}",err));
            return Err(format!("Invalid password: {}", err));
        }

        let refresh_token_data = RefreshToken {
            id: login_data.id,
        };
        
        match generate_refresh_token(refresh_token_data) {
            Ok(refresh_token) => {
                Logger::info_logger(handler_name, log_id, "login_services.generate_refresh_token");
                match UserQuery::create_refresh_token(&refresh_token, login_data.id, db_pool).await {
                    Ok(_) => {
                        let access_token_data = AccessToken {
                            id: login_data.id,
                            email: login_data.email.clone(),
                            username: login_data.username.clone(),
                        };
                        
                        Logger::info_logger (handler_name, log_id, "login_service.save_refresh_token");

                        match generate_access_token(access_token_data) {
                            Ok(access_token) => {
                                let payload = LoginPayload {
                                    id: login_data.id,
                                    email: login_data.email,
                                    username: login_data.username,
                                    access_token,
                                    refresh_token: refresh_token.clone(),
                                };

                                let _ = Self::refresh_token(log_id,refresh_token, db_pool, redis_pool);

                                Ok(payload)
                            }
                            Err(error) => {
                                Logger::warning_logger(handler_name, log_id, "login_service.generate_access_token",&error);
                                Err(format!("Error generating access token: {}", error))
                            },
                        }
                    }
                    Err(error) => {
                        Logger::warning_logger(handler_name, log_id, "login_services.save_refresh_token", &error);
                        Err(format!("Error saving refresh token to database: {}", error))
                    },
                }
            }
            Err(error) => {
                Logger::warning_logger(handler_name, log_id, "login_service.generate_refresh_token", &error);
                Err(format!("Error generating refresh token: {}", error))
            },
        }
    }    

    pub async fn refresh_token(
        log_id: &str,
        token: String,
        db_pool: &DbPool,
        redis_pool: &RedisPool
    )->Result<String,String>{
        let handler_name = "refresh_token";
        let decode_token = decode_refresh_token(&token).map_err(|err|{
            if err.contains("InvalidSignature"){
                let err_message = String::from("error input: invalid token");
                Logger::warning_logger(handler_name, log_id, "refresh_token.decode_token", &err_message);
                return err_message
            }
            let err_message=   format!("error decode token: {}",err);
            Logger::warning_logger(handler_name, log_id, "refresh_token.decode_token", &err_message);

            return err_message
        })?;

        let user_id = UserQuery::find_refresh_token(token, decode_token.claims.token.id, db_pool).await.map_err(|err|{
            let err_message= format!("error find refresh token: {}",err);

            Logger::warning_logger(handler_name, log_id, "refresh_token.find_refresh_token", &err_message);
            return err_message
        })?;

        let user = UserQuery::find_user_by_id(user_id, db_pool).await.map_err(|err|{
            let err_message= format!("error find user: {}",err); 
           
           Logger::warning_logger(handler_name, log_id, "refresh_token.validate_user_id", &err_message);
           return err_message
        })?;

        let access_token = generate_access_token(user).map_err(|err|{
            let err_message= format!("error generate access token: {}",err);
            
            Logger::warning_logger(handler_name, log_id, "refresh_token.generate_access_token", &err_message);

            return err_message
        })?;

        let _ = Self::delete_access_token(redis_pool);

        let _ = Self::store_access_token(access_token.clone(), redis_pool).map_err(|err|{
            let err_message = format!("error store access token: {}",err);

            Logger::warning_logger(&handler_name, log_id, "refresh_token.store_access_token", &err_message);
            return err_message
        })?;

        Ok(access_token)
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
        )-> (){
            let redis_key = format!("access_token");

            let mut conn = create_redis_connection(redis_pool).expect("failed to connect redis");

            let set_data: Result<String, RedisError> = conn.del(&redis_key);
            match set_data {
                Ok(data)=>{
                    info!("data delete: {}",data);
                },
                Err(error)=>{
                    if error.to_string() != "response was int(0)"{
                        info!("key not found");
                    }
                    info!("{}",format!("error redis: {}", error))
                }
            }    
    }

    pub async fn find_user_login(
        log_id: &str,
        token: AccessToken,
        db_pool: &DbPool
    )-> Result<AccessToken,String>{
        let handler_name = "find_user_services";
        match UserQuery::find_user_by_id(token.id, db_pool).await{
            Ok(user)=>{
                Logger::debug_logger(handler_name, log_id, &token, "find_user_services.validate_token", &user);
                if user.email != token.email{
                    return Err("invalid token".to_string());
                }else{
                    Ok(user)
                }
            },
            Err(error)=>{
                Logger::warning_logger(handler_name, log_id, "find_user_services.validate_token", &error);
                return Err(error)
            }
        }
    }

    //register queue
    pub async fn register(
        log_id:&str,
        mut data: RegisterData,
        db_pool: &DbPool,
        rabbit_pool: &RabbitMqPool,
    ) -> Result<RegisterPayload, String> {
        let argon2 = Argon2::default();
        let salt: SaltString = SaltString::generate(&mut OsRng);
        let handler_name= "register_service";
        let password_hash = match argon2.hash_password(data.password.as_bytes(), &salt) {
            Ok(hash) => {
                Logger::info_logger(handler_name,log_id, "register.hash_password");
                hash.to_string()
            },
            Err(e) => {
                Logger::warning_logger(handler_name,log_id, "register.hash_password", &format!("{}",e));
                return Err(format!("Password hash error: {}", e))
            }
        };

        data.password = password_hash;

        if let Err(err)= data.phone_number.parse::<i128>(){
            Logger::warning_logger(handler_name,log_id, "register.parse_phone", &format!("{}",err));
            return Err(format!("input error: {}",err));
        }

        let conn = match rabbit_pool.get().await {
            Ok(conn) => {
                Logger::info_logger(handler_name,log_id, "register.get_rabbit_connections");
                conn
            },
            Err(err) => {
                Logger::err_logger(handler_name,log_id, "register.get_rabbit_connections", &err);
                return Err(format!("RabbitMQ connection error: {}", err));
            }
        };

        let channel: lapin::Channel = match conn.create_channel().await {
            Ok(channel) => {
                Logger::info_logger(handler_name,log_id, "register.create_rmq_channel");
                channel
            },
            Err(err) => {
                Logger::err_logger(handler_name,log_id, "register.create_rmq_channel", &err);
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
            Logger::err_logger(handler_name,log_id, "register.create_queue", &err);
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
                confirm
            },
            Err(err) => {
                return Err(format!("RabbitMQ publish error: {}", err));
            }
        };

        info!("✅ Successfully published register request to queue");

        match UserQuery::create_user(data.clone(), db_pool).await {
            Ok(register_payload) => {
                Logger::debug_logger(handler_name,log_id, &data, "register.create_user", &register_payload);
                Logger::info_logger(handler_name,log_id, "register.create_user");
                Ok(register_payload)
            },
            Err(error) => {
                Logger::warning_logger(handler_name,log_id, "register.create_user", &format!("{}",error));
                Err(format!("{}", error))
        },
        }
    }
}