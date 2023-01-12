// An implementation of user authentication using the http-auth-basic package
// https://docs.rs/http-auth-basic/0.3.3/http_auth_basic/
// transmits user credentials as a PLAINTEXT key-value pair, so it is not secure
// under many circumstances
// https://www.rfc-editor.org/rfc/rfc7617
// https://www.ietf.org/rfc/rfc2617.txt

use std::fmt::Display;

use actix_web::{web::{ServiceConfig, post, Json, self}, Responder, error::{ErrorBadRequest, ErrorInternalServerError, ErrorUnauthorized}, http::{header::TryIntoHeaderValue}, Error, dev::ServiceRequest, FromRequest};
use actix_web_httpauth::{headers::authorization::Basic, extractors::basic::BasicAuth, middleware::HttpAuthentication};
use futures_util::Future;
use serde::{Serialize, Deserialize};

use crate::core::{auth::{Authenticator, AuthenticationError}, service_provider::ServiceProvider};

/*
fn foo<T, F, O>(t: F) -> HttpAuthentication<T, F>
where
    T: FromRequest,
    F: Fn(ServiceRequest, T) -> O, 
    O: Future<Output = Result<ServiceRequest, (Error, ServiceRequest)>>
{
    HttpAuthentication::basic(authentication_middleware)
}
*/

/*
fn bar() -> impl Fn(u32) -> String {
    baz
}

fn baz(i: u32) -> String {
    "hi".to_string()
}

fn qux<T>() -> T
where
    T: Fn(u32) -> String
{
    baz
}
*/

fn foo<F, O>(t: F) -> HttpAuthentication<BasicAuth, F>
where
    F: Fn(ServiceRequest, BasicAuth) -> O,
    O: Future<Output = Result<ServiceRequest, (Error, ServiceRequest)>>
{
    let doo = HttpAuthentication::basic(t);
    doo
}

fn jjj() {
    let m = foo(basic_auth_middleware);
}


async fn basic_auth_middleware(req: ServiceRequest, credentials: BasicAuth) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // can access data through request
    // https://stackoverflow.com/a/64058241
    let sp = req.app_data::<web::Data<ServiceProvider>>().unwrap();
    
    Ok(req)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    username: String,
    password: String // plaintext
}

impl User {
    pub fn create(username: &str, password: &str) -> Result<Self, UserValidationError> {
        let username_valid = username.chars().all(|ch| ch.is_alphanumeric());
        let password_valid = password.chars().all(|ch| ch.is_alphanumeric());

        if username_valid && password_valid {
            Ok(Self {
                username: username.to_string(),
                password: password.to_string()
            })
        } else if !username_valid{
            Err(UserValidationError::InvalidUsername(username.to_string()))
        } else {
            Err(UserValidationError::InvalidPassword(password.to_string()))
        }
    }
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:*******", self.username)
    }
}

#[derive(Debug)]
pub enum UserValidationError {
    InvalidUsername(String),
    InvalidPassword(String)
}

impl Display for UserValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUsername(username) => write!(f, "Invalid username: {}", username),
            Self::InvalidPassword(password) => write!(f, "Invalid password: {}", password)
        }
    }
}

pub struct BasicAuthenticator {

}

impl BasicAuthenticator {
    pub fn new() -> Self {
        Self {

        }
    }
}

impl Authenticator for BasicAuthenticator {
    fn authenticate(&self, http_authorization_header: &str) -> Result<bool, AuthenticationError> {
        println!("BasicAuthenticator: {}", http_authorization_header);
        Ok(true)
    }
}

pub fn configure_basic_authentication_routes(cfg: &mut ServiceConfig) {
    cfg.route("/signup", post().to(signup_handler));
    cfg.route("/signin", post().to(signin_handler));
}

async fn signup_handler(credentials: Json<User>) -> impl Responder {
    let maybe_valid_user = User::create(
        &credentials.username, 
        &credentials.password
    );

    if let Err(user_validation_error) = maybe_valid_user {
        return Err(ErrorBadRequest(user_validation_error));
    }

    Ok(Json(maybe_valid_user.unwrap()))
}

async fn signin_handler(credentials: Json<User>) -> Result<String, Error> {
    let basic_credentials = Basic::new(
        credentials.username.to_owned(), 
        Some(credentials.password.to_owned())
    );

    let maybe_header = basic_credentials.try_into_value();
    if maybe_header.is_err() {
        return Err(ErrorBadRequest(credentials));
    }

    let header = maybe_header.unwrap();
    let maybe_str = header.to_str();
    match maybe_str {
        Ok(as_str) => Ok(as_str.to_owned()),
        Err(e) => Err(ErrorInternalServerError(e))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn usernames_cannot_contain_colon() {
        let username = "Username:123";
        let result = User::create(username, "password");
        assert!(result.is_err());
        match result.unwrap_err() {
            UserValidationError::InvalidUsername(invalid_name) => assert_eq!(username, invalid_name),
            _ => panic!("Expected InvalidUsername")
        };
    }

    #[test]
    fn alphanumeric_usernames_are_allowed() {
        let username = "Username123";
        let result = User::create(username, "password");
        assert!(result.is_ok());
        assert_eq!(username, result.unwrap().username);
    }
}