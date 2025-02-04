use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHasher};
use pgsql_libs::DbPool;

use super::{user_models::{RegisterData, RegisterPayload}, user_query::UserQuery};



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
}