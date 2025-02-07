use std::fmt;

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, http::header::HeaderValue, Error, HttpMessage, HttpResponse, ResponseError
};
use futures::future::{ok, LocalBoxFuture, Ready};
use serde_json::json;

#[derive(Debug)]
pub struct UnauthorizedError;

impl fmt::Display for UnauthorizedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unauthorized: Invalid or missing token")
    }
}

impl ResponseError for UnauthorizedError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::Unauthorized()
            .json(json!({"error": "Unauthorized", "message": self.to_string()}))
    }
}
pub struct RefreshTokenMW;

impl<S, B> Transform<S, ServiceRequest> for RefreshTokenMW
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Transform = RefreshTokenMiddleware<S>;
    type Error = Error; 
    type InitError = (); 
    type Future = Ready<Result<Self::Transform, Self::InitError>>; 

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RefreshTokenMiddleware { service })
    }
}

pub struct RefreshTokenMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RefreshTokenMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>; 
    type Error = Error; 
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>; 

    forward_ready!(service);    

    fn call(&self, req: ServiceRequest) -> Self::Future {
        
        let refresh_token: Option<&HeaderValue> = req.headers().get("refresh-token");

        if let Some(token) = refresh_token {
          
            if let Ok(token_str) = token.to_str() {
                
                req.extensions_mut().insert(token_str.to_string());

                let fut = self.service.call(req);
                return Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                });
            }
        }

        Box::pin(async {Err(UnauthorizedError.into())})
}

}