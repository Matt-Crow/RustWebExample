// this file demonstrates how to use JSON web tokens to authenticate users

use std::env;

use actix_web::{dev::ServiceRequest, Error, error::{ErrorInternalServerError, ErrorUnauthorized}, web::{ServiceConfig, post, Json}};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, EncodingKey, Header, decode, DecodingKey, Validation, Algorithm};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use common::user::User;

const ISSUER: &str = "https://example.com";

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    iss: String, // issuer
    sub: String, // subject
    aud: String, // receiving audience 
    exp: i64,    // expiration datetime, in seconds from start of 1970
    iat: i64,    // issued datetime, as per exp
    user: User   // custom claim
}

pub fn configure_jwt_routes(cfg: &mut ServiceConfig) {
    cfg.route("/jwt", post().to(jwt_login_handler));
}

async fn jwt_login_handler(user: Json<User>) -> actix_web::Result<String> {
    // a production system would verify the user's credentials
    match make_token(&user.0) {
        Ok(token) => Ok(token),
        Err(e) => Err(ErrorInternalServerError(e))
    }
} 

/// creates a JWT for the given user
pub fn make_token(user: &User) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let later = now.checked_add_signed(Duration::minutes(30)).unwrap();

    let claims = Claims {
        iss: String::from(ISSUER),
        sub: user.email(),
        aud: String::from(ISSUER),
        exp: later.timestamp(),
        iat: now.timestamp(),
        user: user.to_owned()
    };
    
    encode(
        &Header::default(), 
        &claims, 
        &EncodingKey::from_secret(load_secret().as_ref())
    )
}

fn load_secret() -> String {
    env::var("JWT_SECRET").expect("Don't forget to set the JWT_SECRET environment variable!")
}

/// Usage: wrap(HttpAuthentication::Bearer(jwt_auth_middleware))
pub async fn jwt_auth_middleware(
    request: ServiceRequest, 
    bearer: BearerAuth
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    match decode_token(bearer.token()) {
        Ok(claims) => {
            println!("Claims: {:#?}", claims);
            println!("Request: {:#?}", request);

            // check if any of the user's groups are authorized to perform the request
            if is_get(&request) || claims.user.groups().iter().any(|g| is_group_authorized(g, &request)) {
                println!("Authorized.");
                Ok(request)
            } else {
                println!("Unauthorized.");
                Err((ErrorUnauthorized("You do not belong to any group authorized to access this resource"), request))
            }
        },
        Err(e) => Err((ErrorInternalServerError(e), request))
    }
}

fn decode_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let mut validator = Validation::new(Algorithm::HS256);
    validator.set_audience(&[ISSUER]); // reject if audience doesn't match
    validator.set_issuer(&[ISSUER]); // reject if issuer doesn't match

    // automatically validates the expiration date
    let result = decode::<Claims>(
        token, 
        &DecodingKey::from_secret(load_secret().as_ref()), 
        &validator
    );

    match result {
        Ok(token_data) => Ok(token_data.claims),
        Err(e) => Err(e)
    }
}

fn is_get(request: &ServiceRequest) -> bool {
    request.method() == Method::GET
}

fn is_group_authorized(group: &str, _request: &ServiceRequest) -> bool {
    group == "admin"
}