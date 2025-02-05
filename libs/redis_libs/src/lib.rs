use r2d2_redis::{r2d2::Pool, RedisConnectionManager};

pub type RedisPool = Pool<RedisConnectionManager>;

pub fn redis_connect(hostname:String,password:Option<String>) -> RedisPool{
    let redis_password = match password {
        Some(pwd) => pwd,
        None => String::new(),
    };

    let conn_url = format!("redis://{}@{}",redis_password,hostname);

    let manager: RedisConnectionManager = RedisConnectionManager::new(conn_url).expect("Invalid connection URL");

    Pool::builder()
        .min_idle(Some(5))
        .max_size(50) 
        .build(manager)
        .expect("Failed to create Redis connection pool")
}