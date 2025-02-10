use config::{Config, File, Environment};
use serde::de::DeserializeOwned;
use dotenv::dotenv;
use std::fmt::Debug;

pub fn libs_config<T>(config_path: &str,prefix: &str) -> T
where
    T: DeserializeOwned + Debug + Default + PartialEq + Eq,  
{
    dotenv().ok();

    let settings = Config::builder()
        .add_source(File::with_name(config_path))
        .add_source(
            Environment::with_prefix(prefix)
                .separator("_"),
        )
        .build()
        .unwrap();

    let deserialized: Result<T, config::ConfigError> = settings.try_deserialize();

    match deserialized {
        Ok(config)=>{
            return config
        },
        Err(error)=>{
            println!("error config: {:?}",error);
            std::process::exit(1)
        }
    }
}
