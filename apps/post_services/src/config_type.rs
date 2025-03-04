use serde::Deserialize;

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Database{
    pub user: String,
    pub password: String,
    pub name: String,
    pub port: u64,
    pub host: String,
    pub url: String,
    pub min_pool_connection: u32,
    pub max_pool_connection: u32
}

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Apps{
    pub address: String
}

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Redis{
    pub host: String,
    pub min_pool_connection: u32,
    pub max_pool_connection: u32
}

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Logger{
    log: String
}

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
pub struct PostAppConfig{
    pub apps: Apps,
    pub database: Database,
    pub redis: Redis,
    pub logger: Logger
}
