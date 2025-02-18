use std::fmt::Debug;
use log::{debug, error, info, warn};
use regex::Regex;
use serde_json::{from_str, json, Value};
use serde::Serialize;

pub fn json_conferter<T>(data:T)
->Option<serde_json::Map<String, Value>>
where 
    T:Serialize
{
    let mut request_json = serde_json::to_value(data).unwrap_or(json!({}));

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
        if let Some(phone_number) = obj.get_mut("phone_number") {
            *phone_number = Value::String(phone_mask(phone_number.as_str().unwrap_or("")));
        }
    }

    request_json.as_object().cloned()
    }

pub fn debug_logger<T, B>(log_id: &str, handler: &str,method: &str, request: &T, response: &B)
where
    T: Serialize + Debug,
    B: Serialize + Debug,
{
    let request = json_conferter(request);

    let response=  json_conferter(response);

    debug!(
        "[ {} ] {}.{} Request: {} | Response: {}",
        log_id,
        handler,
        method,
        format!("{:?}", request.unwrap()),
        format!("{:?}", response.unwrap())
    );
}

pub fn info_logger(
    log_id: &str,
    handler: &str,
    method: &str
)
{
    info!(
        "[ {} ] {}.{}",
        log_id,
        handler,
        method
    )
}


pub fn warning_logger(log_id: &str, handler: &str, method: &str, message: &str) {
    let re = Regex::new(r#"Bytes\((b?"(.*)")\)"#).unwrap();
    
    let extracted_message = if let Some(caps) = re.captures(message) {
        caps.get(1).map_or(message, |m| m.as_str())
    } else {
        message
    };

    let cleaned_message = extracted_message
        .trim_start_matches("b\"")
        .trim_start_matches('"')
        .trim_end_matches('"')
        .trim_end_matches("\"");

    let unescaped_json: String = match from_str(cleaned_message) {
        Ok(json_str) => json_str,
        Err(_) => cleaned_message.to_string(),
    };

    let formatted_message = match from_str::<Value>(&unescaped_json) {
        Ok(json) => json.to_string(),
        Err(_) => {
            unescaped_json 
        },
    };

    warn!(
        "[ {} ] {}.{} warning: {}",
        log_id, handler, method, formatted_message
    );
}


pub fn error_logger(
    method: &str,
    message: &str,
)
{
    error!(
        "{}, error: {}",
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