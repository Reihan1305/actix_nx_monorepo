use std::fmt::Debug;
use log::{debug, error, info, warn};
use serde_json::{json, Value};
use serde::Serialize;

pub fn json_conferter<T>(data:T)
->Option<serde_json::Map<String, Value>>
where 
    T:Serialize
{
    let mut request_json = serde_json::to_value(data).unwrap_or(json!({}));

    if let Some(obj) = request_json.as_object_mut() {
        if let Some(username) = obj.get_mut("username") {
            *username = Value::String(MaskData::username_mask(username.as_str().unwrap_or("")));
        }
        if let Some(password) = obj.get_mut("password") {
            *password = Value::String(MaskData::password_mask());
        }
        if let Some(msisdn) = obj.get_mut("msisdn") {
            *msisdn = Value::String(MaskData::phone_mask(msisdn.as_str().unwrap_or("")));
        }
        if let Some(phone_number) = obj.get_mut("phone_number") {
            *phone_number = Value::String(MaskData::phone_mask(phone_number.as_str().unwrap_or("")));
        }
    }
    request_json.as_object().cloned()
}
pub struct Logger;

impl Logger {
    pub fn debug_logger <T, B>(
        handler: &str,
        log_id: &str,
        request: &T,
        title:&str,
        response: &B,
    ) where
        T: Serialize + Debug,
        B: Serialize + Debug,
    {
        let request = json_conferter(request);

        let response=  json_conferter(response);

        debug!(
            target:handler,
            "{{{} | {}}} Request: {} | Response: {}",
            log_id,
            title,
            format!("{:?}", request.unwrap()),
            format!("{:?}", response.unwrap())
        );
    }

    pub fn info_logger(
        handler:&str,
        log_id: &str,
        title: &str
    ) {
        info!(target:handler,
            "{{{} | {}}}",
            log_id,
            title
        )
    }

    pub fn warning_logger(
        handler: &str,
        log_id: &str,
        title: &str,
        message: &str,
    ) {
        warn!(
            target:handler,
            "{{{} | {}}} {}",
            log_id,title, message
        );
    }

    pub fn err_logger <T>(
        handler: &str,
        log_id: &str,
        title: &str,
        error: T,
    ) where
        T: Debug,
    {
        error!(
            target:handler,
            "{{{} | {}}} message: {:?}",
            log_id, title, error
        );
    }
}

pub struct MaskData;

impl MaskData{
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
}