// provides the core details required for authentication & authorization

use actix_web::{dev::ServiceRequest, Error};
use actix_web_httpauth::extractors::basic::BasicAuth;

/// marks a struct as providing authentication of HTTP requests
pub trait Authenticator {
    /// checks the result of a request's authorization header, then returns true
    /// if it is valid and no errors occur
    fn authenticate(http_authorization_header: &str) -> Result<bool, AuthenticationError>;
}

pub struct AuthenticationError(String);

pub async fn authentication_middleware(req: ServiceRequest, _credentials: BasicAuth) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    
    Ok(req)
}