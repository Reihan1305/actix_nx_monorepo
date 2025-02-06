use std::sync::Arc;
use jwt_libs::decode_access_token;
use r2d2_redis::redis::Commands;
use tonic::{Request, Status};
use redis_libs::RedisPool;

#[derive(Clone)]
pub struct AuthMiddleware {
    redis_pool: Arc<RedisPool>,
}

impl AuthMiddleware {
    pub fn new(redis_pool: Arc<RedisPool>) -> Self {
        Self { redis_pool }
    }

    pub fn auth_check(&self, mut req: Request<()>) -> Result<Request<()>, Status> {
        let mut redis_conn = match self.redis_pool.get() {
            Ok(conn) => conn,
            Err(error) => return Err(Status::internal(format!("Redis error: {}", error))),
        };

        let redis_key = String::from("access_token");

        let token:String  = match redis_conn.get::<String,String>(redis_key) {
            Ok(token) => token,
            Err(_) => return Err(Status::unauthenticated("Invalid token: not found in Redis")),
        };

        // **Perbaikan: Menghapus titik koma di bawah ini**
        match decode_access_token(&token) {
            Ok(decoded_token) => {
                let access_token = decoded_token.claims.token;
                println!("Decoded Access Token: {:?}", access_token);

                req.extensions_mut().insert(Arc::new(access_token));
                Ok(req) // **Jangan pakai titik koma (;)**
            }
            Err(error) => Err(Status::unauthenticated(format!("Invalid token: {}", error))),
        } // **Jangan pakai titik koma di sini**
    }
}
