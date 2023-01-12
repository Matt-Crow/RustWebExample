// provides the core details required for authentication & authorization

use std::{rc::Rc, future::{Ready, ready}, sync::Arc, fmt::Display};
use actix_web::{dev::{ServiceRequest, Service, ServiceResponse, forward_ready, Transform}, Error, error::{ErrorUnauthorized, ErrorBadRequest}};
use futures_util::{future::LocalBoxFuture, FutureExt};

pub fn make_actix_authenticator(auth: Arc<dyn Authenticator>) -> ActixMiddlewareAdapterFactory {
    ActixMiddlewareAdapterFactory::new(Arc::new(AuthenticationMiddlewareAdapter::new(auth)))
}

/// marks a struct as providing authentication of HTTP requests
pub trait Authenticator: Send + Sync { // must be safe for multiple threads to access at the same time
    /// checks the result of a request's authorization header, then returns true
    /// if it is valid and no errors occur
    fn authenticate(&self, http_authorization_header: &str) -> Result<bool, AuthenticationError>;
}

#[derive(Debug)]
pub struct AuthenticationError(String);

impl Display for AuthenticationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Authentication error: {}", self.0)
    }
}

pub struct AuthenticationMiddlewareAdapter {
    authenticator: Arc<dyn Authenticator>
}

impl AuthenticationMiddlewareAdapter {
    pub fn new(authenticator: Arc<dyn Authenticator>) -> Self {
        Self {
            authenticator
        }
    }

    fn extract_auth_header(&self, req: &ServiceRequest) -> Result<String, Error> {
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
}

impl MiddlewareAdapter for AuthenticationMiddlewareAdapter {
    fn apply_to(&self, req: &ServiceRequest) -> Result<(), Error> {
        let auth_header = self.extract_auth_header(req)?;
        let is_auth = self.authenticator.authenticate(&auth_header)
            .map_err(ErrorUnauthorized)?;
        if is_auth {
            Ok(())
        } else {
            Err(ErrorUnauthorized("You are not authorized to access this resource"))
        }
    }
}

/// the adapter design pattern makes it easier to adapt our needs to Actix's
/// complicated interface
pub trait MiddlewareAdapter {
    /// checks the current request, and should return an error if Actix should
    /// stop processing the request
    fn apply_to(&self, req: &ServiceRequest) -> Result<(), Error>;
}

pub struct ActixMiddlewareAdapterFactory {
    adapter: Arc<dyn MiddlewareAdapter>
}

impl ActixMiddlewareAdapterFactory {
    pub fn new(adapter: Arc<dyn MiddlewareAdapter>) -> Self {
        Self {
            adapter
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ActixMiddlewareAdapterFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ActixMiddlewareAdapter<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ActixMiddlewareAdapter {
            adapted: self.adapter.clone(),
            actix_service: Rc::new(service)
        }))
    }
}

pub struct ActixMiddlewareAdapter<S> {
    adapted: Arc<dyn MiddlewareAdapter>,
    actix_service: Rc<S>
}

impl<S, B> Service<ServiceRequest> for ActixMiddlewareAdapter<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(actix_service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.actix_service.clone();
        let adapted = self.adapted.clone();

        async move {
            adapted.apply_to(&req)?;
            let res = service.call(req).await?;
            Ok(res)
        }.boxed_local()
    }
}