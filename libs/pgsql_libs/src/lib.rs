use sqlx::{Error, Pool, Postgres, postgres::PgPoolOptions};

pub async fn create_db_pool(db_url:String, min_con:u32, max_con:u32)-> Result<Pool<Postgres>,Error>{
    let db_pool: Result<Pool<Postgres>, sqlx::Error> = match PgPoolOptions::new()
        .min_connections(min_con)
        .max_connections(max_con)
        .connect(&db_url)
        .await {
            Ok(pg_pool)=> Ok(pg_pool),
            Err(err) => Err(err)
        };
    db_pool
}

pub type DbPool = Pool<Postgres>;