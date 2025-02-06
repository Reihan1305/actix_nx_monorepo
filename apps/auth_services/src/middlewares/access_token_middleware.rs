use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, 
    web::Data,
    Error, 
    HttpMessage, 
};
use futures::future::{ok, LocalBoxFuture, Ready};
use r2d2_redis::redis::Commands;

use  crate::AppState;
use jwt_libs::decode_access_token;
use super::refresh_token_middleware::UnauthorizedError;

pub struct AccessTokenMW;

impl<S, B> Transform<S, ServiceRequest> for AccessTokenMW
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>; 
    type Transform = AccessTokenMiddleware<S>; 
    type Error = Error;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AccessTokenMiddleware { service })
    }
}

pub struct AccessTokenMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AccessTokenMiddleware<S>
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
        let app_state: Option<&Data<AppState>> = req.app_data::<Data<AppState>>();

        if let Some(state) = app_state {
            let mut redis_conn = match state.redis.get() {
                Ok(conn) => conn,
                Err(_) => return Box::pin(async { Err(UnauthorizedError.into()) }),
            };

            let redis_key = "access_token".to_string();

            match redis_conn.get::<String, String>(redis_key) {
                Ok(token) => {
                    match decode_access_token(&token) {
                        Ok(user) => {
                            req.extensions_mut().insert(user.claims.token);

                            let fut = self.service.call(req);
                            return Box::pin(async move {
                                let res = fut.await?;
                                Ok(res)
                            });
                        }
                        Err(_) => {
                            return Box::pin(async { Err(UnauthorizedError.into()) });
                        }
                    }
                }
                Err(_) => {
                    return Box::pin(async { Err(UnauthorizedError.into()) });
                }
            }
        }

        Box::pin(async { Err(UnauthorizedError.into()) })
    }
}