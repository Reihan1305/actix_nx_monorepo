use r2d2_redis::{r2d2::{self, Pool, PooledConnection}, RedisConnectionManager};

pub type RedisPool = Pool<RedisConnectionManager>;

pub fn redis_connect(hostname:String,password:Option<String>,min_con:u32, max_conn:u32) -> Result<Pool<RedisConnectionManager>,r2d2::Error>{
    let redis_password = match password {
        Some(pwd) => pwd,
        None => String::new(),
    };

    let conn_url = format!("redis://{}@{}",redis_password,hostname);

    let manager: RedisConnectionManager = RedisConnectionManager::new(conn_url).expect("Invalid connection URL");

    match Pool::builder()
        .min_idle(Some(min_con))
        .max_size(max_conn) 
        .build(manager){
            Ok(redis_pool)=>Ok(redis_pool),
            Err(err) =>Err(err)
        }
}

pub fn create_redis_connection(redis_pool: &RedisPool) -> Result<PooledConnection<RedisConnectionManager>, r2d2::Error>{
    match redis_pool.get(){
        Ok(conn) => Ok(conn),
        Err(error) => Err(error),
    }
}