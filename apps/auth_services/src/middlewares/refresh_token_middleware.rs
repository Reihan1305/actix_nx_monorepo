use std::fmt;

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::HeaderValue,
    Error, HttpMessage, HttpResponse, ResponseError,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use serde_json::json; // Untuk JSON response

#[derive(Debug)]
struct UnauthorizedError;

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


// ✅ Middleware Factory Struct
pub struct RefreshToken;

impl<S, B> Transform<S, ServiceRequest> for RefreshToken
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>; // Jenis response middleware
    type Transform = RefreshTokenMiddleware<S>; // Middleware yang dibuat
    type Error = Error; // Jenis error yang dihasilkan
    type InitError = (); // Tidak ada error saat inisialisasi
    type Future = Ready<Result<Self::Transform, Self::InitError>>; // Future yang langsung siap

    fn new_transform(&self, service: S) -> Self::Future {
        // ✅ Factory untuk membuat middleware baru
        ok(RefreshTokenMiddleware { service })
    }
}

// ✅ Middleware Struct
pub struct RefreshTokenMiddleware<S> {
    service: S, // Service yang di-wrap oleh middleware
}

impl<S, B> Service<ServiceRequest> for RefreshTokenMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>; // Response yang dikembalikan oleh middleware
    type Error = Error; // Jenis error yang bisa terjadi
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>; // Future untuk response

    forward_ready!(service); // Meneruskan status ready ke service yang di-wrap

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // ✅ Mengambil header "refresh-token"
        let refresh_token: Option<&HeaderValue> = req.headers().get("refresh-token");

        if let Some(token) = refresh_token {
            // ✅ Konversi HeaderValue ke string
            if let Ok(token_str) = token.to_str() {
                // ✅ Menyimpan token ke dalam request extension agar bisa diakses di handler
                req.extensions_mut().insert(token_str.to_string());

                // ✅ Memproses request lebih lanjut dengan middleware
                let fut = self.service.call(req);
                return Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                });
            }
        }

        // ✅ Jika token tidak ditemukan atau tidak valid, kembalikan error Unauthorized
        Box::pin(async {Err(UnauthorizedError.into())})
}

}