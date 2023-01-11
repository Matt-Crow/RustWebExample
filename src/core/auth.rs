use std::{rc::Rc, future::{Ready, ready}};

// provides the core details required for authentication & authorization
use actix_web::{dev::{ServiceRequest, Service, ServiceResponse, forward_ready, Transform}, Error, web, error::{ErrorUnauthorized, ErrorBadRequest, ErrorInternalServerError}};
use actix_web_httpauth::extractors::basic::BasicAuth;
use futures_util::{future::LocalBoxFuture, FutureExt};

use super::service_provider::ServiceProvider;

/// marks a struct as providing authentication of HTTP requests
pub trait Authenticator {
    /// checks the result of a request's authorization header, then returns true
    /// if it is valid and no errors occur
    fn authenticate(&self, http_authorization_header: &str) -> Result<bool, AuthenticationError>;
}

pub struct AuthenticationError(String);

/// built on top of the Actix Web BasicAuth crate, which is not ideal
pub async fn authentication_middleware(req: ServiceRequest, _credentials: BasicAuth) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let authentication_header = req.headers().get("Authorization");
    if authentication_header.is_none() {
        return Err((
            ErrorUnauthorized("Missing Authorization header"),
            req
        ));
    }

    let maybe_auth_value = authentication_header.unwrap().to_str();
    if maybe_auth_value.is_err() {
        return Err((
            ErrorBadRequest("Invalid Authentication header"),
            req
        ));
    }
    
    let auth_value = maybe_auth_value.unwrap();
    // https://stackoverflow.com/a/64058241
    let sp = req.app_data::<web::Data<ServiceProvider>>().unwrap();

    match sp.authenticator().authenticate(auth_value) {
        Ok(is_authentic) => {
            if is_authentic {
                Ok(req)
            } else {
                Err((
                    ErrorUnauthorized(""),
                    req
                ))
            }
        },
        Err(_) => Err((
            ErrorInternalServerError(""),
            req
        ))
    }
}

pub struct AuthenticationMiddlewareFactory {

}

impl AuthenticationMiddlewareFactory {
    pub fn new() -> Self {
        Self {

        }
    }
}

impl Default for AuthenticationMiddlewareFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthenticationMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware {
            service: Rc::new(service)
        }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: Rc<S>
}

impl<S> AuthenticationMiddleware<S> {
    fn extract_auth_header(req: &ServiceRequest) -> Result<String, Error> {
        let maybe_auth_header = req.headers().get("Authorization");
        if maybe_auth_header.is_none() {
            return Err(ErrorUnauthorized("Missing Authorization header"));
        }

        let maybe_header_value = maybe_auth_header.unwrap().to_str();
        if maybe_header_value.is_err() {
            return Err(ErrorBadRequest("Invalid Authorization header"));
        }

        let header_value = maybe_header_value.unwrap();

        Ok(header_value.to_owned())
    }

    fn is_authentic(req: &ServiceRequest, h: &str) -> Result<bool, Error> {
        let maybe_service = req.app_data::<web::Data<ServiceProvider>>();
        match maybe_service {
            Some(service) => match service.authenticator().authenticate(h) {
                Ok(b) => Ok(b),
                Err(_) => Err(ErrorInternalServerError("Failed to authenticate"))
            },
            None => Err(ErrorInternalServerError("Failed to load ServiceProvider"))
        }
    }
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
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

        async move {
            let header = Self::extract_auth_header(&req)?;
            let is_authentic = Self::is_authentic(&req, &header)?;
            if is_authentic {
                let res = service.call(req).await?;
                Ok(res)
            } else {
                Err(ErrorUnauthorized("You are not authorized"))
            }
        }.boxed_local()
    }
}