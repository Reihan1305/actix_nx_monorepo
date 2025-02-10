use serde::Deserialize;

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Database{
    pub user: String,
    pub password: String,
    pub name: String,
    pub port: u64,
    pub host: String,
    pub url: String
}

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Apps{
    port:u64
}

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Redis{
    pub host: String
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
