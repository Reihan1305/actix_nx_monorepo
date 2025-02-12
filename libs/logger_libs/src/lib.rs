use std::fmt::Debug;
use log::{debug, error, info, warn};
use serde_json::{Value, json};
use serde::Serialize;

pub fn debug_logger<T, B>(log_id: &str, handler: &str,method: &str, request: &T, response: &B)
where
    T: Serialize + Debug,
    B: Serialize + Debug,
{
    let mut request_json = serde_json::to_value(request).unwrap_or(json!({}));

    if let Some(obj) = request_json.as_object_mut() {
        if let Some(username) = obj.get_mut("username") {
            *username = Value::String(username_mask(username.as_str().unwrap_or("")));
        }
        if let Some(password) = obj.get_mut("password") {
            *password = Value::String(password_mask());
        }
        if let Some(msisdn) = obj.get_mut("msisdn") {
            *msisdn = Value::String(phone_mask(msisdn.as_str().unwrap_or("")));
        }
    }

    let request=  request_json.as_object();
    debug!(
        "[ {} ] {}.{} Request: {} | Response: {}",
        log_id,
        handler,
        method,
        format!("{:?}", request.unwrap()),
        format!("{:?}", response)
    );
}

pub fn info_logger(
    log_id: &str,
    handler: &str,
    method: &str
)
{
    info!(
        "{} {}.{}",
        log_id,
        handler,
        method
    )
}

pub fn warning_logger(
    log_id: &str,
    handler: &str,
    method: &str,
    message: &str
){
    warn!(
        "{} {}.{} warning: {}",
        log_id,
        handler,
        method,
        message
    )
}

pub fn error_logger(
    log_id: &str,
    handler: &str,
    method: &str,
    message: &str,
)
{
    error!(
        "{} {}.{} error: {}",
        log_id,
        handler,
        method,
        message
    )
}

pub fn password_mask()->String{
    String::from("***")
}

pub fn username_mask(username:&str)->String {
    let hashed_user: Vec<String> = username
        .split(' ') 
        .map(|word| {
            let prefix = &word[..std::cmp::min(3, word.len())]; 
            format!("{}***", prefix) 
        })
        .collect();

    let final_username = hashed_user.join(" "); 

    final_username
}

fn phone_mask(msisdn: &str) -> String {
    if msisdn.len() > 5 {
        let first_part = &msisdn[..3];
        let last_part = &msisdn[msisdn.len() - 3..];
        format!("{}***{}", first_part, last_part)
    } else {
        "***".to_string()
    }
}