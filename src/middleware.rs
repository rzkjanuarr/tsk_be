use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error, Error, HttpMessage,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use std::rc::Rc;

use crate::utils::decode_token;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Rc::new(service),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            let auth_header = req.headers().get("Authorization");

            let token = match auth_header {
                Some(header) => {
                    let header_str = header.to_str().unwrap_or("");
                    if header_str.starts_with("Bearer ") {
                        &header_str[7..]
                    } else {
                        return Err(error::ErrorUnauthorized("Invalid authorization header"));
                    }
                }
                None => return Err(error::ErrorUnauthorized("Missing authorization header")),
            };

            let claims = match decode_token(token)
                .map_err(|_| error::ErrorUnauthorized("Invalid or expired token"))?;

            req.extensions_mut().insert(claims.sub.clone());

            let res = service.call(req).await?;
            Ok(res)
        })
    }
}

pub fn get_user_id(req: &actix_web::HttpRequest) -> Result<String, Error> {
    req.extensions()
        .get::<String>()
        .cloned()
        .ok_or_else(|| error::ErrorInternalServerError("User ID not found"))
}
