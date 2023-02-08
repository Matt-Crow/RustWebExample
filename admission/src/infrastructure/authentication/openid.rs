// OpenID defers the authentication of users to a third-party service.
// derived from 
// https://github.com/ramosbugs/openidconnect-rs/blob/main/examples/google.rs
// https://openid.net/specs/openid-connect-basic-1_0.html

use std::{env, fmt::{Display, Debug}, error::Error};

use actix_session::Session;
use actix_web::{web::{ServiceConfig, get, self}, HttpResponse, error};
use openidconnect::{core::{CoreProviderMetadata, CoreClient, CoreResponseType, CoreAuthPrompt}, IssuerUrl, reqwest::async_http_client, ClientId, RedirectUrl, CsrfToken, Nonce, AuthenticationFlow, Scope, ClientSecret, AuthorizationCode};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::core::users::{User, UserService};

use super::jwt::make_token;

/// used by main to set up the openid routes
pub fn configure_openid_routes(cfg: &mut ServiceConfig) {
    cfg.route("/login", get().to(login_handler));
    cfg.route("/openid", get().to(handle_auth_callback));
}

/// handles requests to the login route
async fn login_handler(
    service: web::Data<OpenIdService>,
    session: Session
) -> actix_web::Result<HttpResponse> {

    let url_etc = service.generate_auth_url();
    
    // as a security measure, the user requesting to log in must remember a
    // single-use nonce and state, which will be verified when handling the
    // openid server response
    session.insert("openid-nonce", url_etc.nonce.secret())
        .map_err(error::ErrorInternalServerError)?;
    session.insert("openid-state", url_etc.state.secret())
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::SeeOther() // redirects user to authentication URL
        .append_header(("Location", url_etc.url.to_string()))
        .finish())
}

/// once the user authenticates with OpenID, our app needs to listen for a
/// message from the OpenID server, as it will give us an authorization code
/// we can use to obtain information granted in the scopes
async fn handle_auth_callback(
    service: web::Data<OpenIdService>,
    users: web::Data<Mutex<UserService>>,
    session: Session,
    openid_response: web::Query<AuthenticationCallbackParameters>
) -> actix_web::Result<HttpResponse> {

    let nonce: String = session.get("openid-nonce")
        .map_err(error::ErrorBadRequest)?
        .ok_or_else(|| error::ErrorBadRequest("openid-nonce not set"))?;
    
    let state: String = session.get("openid-state")
        .map_err(error::ErrorBadRequest)?
        .ok_or_else(|| error::ErrorBadRequest("openid-state not set"))?;
    
    session.purge(); // no longer need session

    let email = service.handle_callback(AuthenticationRequest { 
            params: openid_response.0, 
            old_nonce: nonce, 
            old_state: state 
        })
        .await
        .map_err(error::ErrorBadRequest)?;

    let mut lock = users.lock()
        .await;
    
    let new_user = lock.get_user_by_email(&email)
        .await
        .map_err(error::ErrorInternalServerError)?;
    
    let jwt = make_token(&new_user)
        .map_err(error::ErrorInternalServerError)?;
    
    Ok(HttpResponse::Ok().json(CallbackResponse {
        jwt,
        user: new_user
    }))
}

#[derive(Debug, Serialize)]
struct CallbackResponse {
    jwt: String,
    user: User
}

/// provides services related to OpenID
#[derive(Debug)]
pub struct OpenIdService {
    client: CoreClient
}

impl OpenIdService {
    
    /// reads the environment variables and creates a new OpenIdService accordingly
    pub async fn from_env() -> Result<Self, OpenIdError> {
        let options = OpenIdOptions::from_env()?;
        let issuer_url = IssuerUrl::new(options.url.to_owned())
            .map_err(|_| OpenIdError::BadIssuer(options.url.to_owned()))?;
        
        let provider_document = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
            .await
            .expect("Expected issuer to provide a provider document");
    
        let client = CoreClient::from_provider_metadata(
                provider_document,
                ClientId::new(options.client_id),
                options.client_secret.map(ClientSecret::new)
            )
            .set_redirect_uri(
                RedirectUrl::new(String::from("http://localhost:8080/openid")).expect("Expected valid URL")
            );

        Ok(Self {
            client
        })
    }

    /// creates a new authorization URL and associated security parameters    
    fn generate_auth_url(&self) -> OpenIdUrl {
        
        let (authorization_url, csrf_state, nonce) = self.client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode, 
                CsrfToken::new_random, 
                Nonce::new_random
            )
            .add_scope(Scope::new(String::from("email")))
            .add_prompt(CoreAuthPrompt::Consent)
            .url();

        OpenIdUrl {
            url: authorization_url,
            nonce,
            state: csrf_state
        }
    }

    /// called after a user authenticates with the OpenID provider, returns 
    /// email
    async fn handle_callback(
        &self, 
        auth_request: AuthenticationRequest
    ) -> Result<String, OpenIdError> {

        // validate CSRF token        
        if auth_request.old_state != auth_request.params.state {
            return Err(OpenIdError::BadCsrfToken);
        }
        
        // exchange authorization code for claims about the user
        let token_response = self.client
            .exchange_code(AuthorizationCode::new(auth_request.params.code))
            .request_async(async_http_client)
            .await
            .map_err(OpenIdError::trace)?;
        
        //println!("token response: {:#?}", token_response);

        let claims = token_response
            .extra_fields()
            .id_token()
            .expect("should contain ID token")
            .claims(&self.client.id_token_verifier(), &Nonce::new(auth_request.old_nonce))
            .map_err(OpenIdError::trace)?;
        
        let email = claims.email()
            .expect("ID token should contain email")
            .to_string();

        Ok(email)
    }
}

#[derive(Debug)]
pub struct OpenIdOptions {
    url: String,
    client_id: String,
    client_secret: Option<String>
}

impl OpenIdOptions {

    pub fn from_env() -> Result<Self, OpenIdError> {
        let url = env::var("OPENID_URL")
            .map_err(|_| OpenIdError::missing_env("OPENID_URL"))?;
        let id = env::var("OPENID_CLIENT_ID")
            .map_err(|_| OpenIdError::missing_env("OPENID_CLIENT_ID"))?;
        let maybe_secret = env::var("OPENID_CLIENT_SECRET")
            .ok(); // can be None

        Ok(OpenIdOptions { 
            url, 
            client_id: id, 
            client_secret: maybe_secret 
        })
    }
}

#[derive(Debug)]
pub enum OpenIdError {
    Other(String),
    MissingEnv(String),
    BadCsrfToken,
    BadIssuer(String)
}

impl OpenIdError {
    fn other(obj: impl ToString) -> Self {
        Self::Other(obj.to_string())
    }

    fn trace(err: impl Error + 'static) -> Self {
        let mut msg = String::from("Trace:");
        let mut curr: Option<&(dyn Error + 'static)> = Some(&err);
        while curr.is_some() {
            msg.push_str(&format!("\nCaused by: {:#?}", curr));
            curr = curr.unwrap().source();
        }
        Self::other(msg)
    }

    fn missing_env(name: &str) -> Self {
        Self::MissingEnv(String::from(name))
    }
}

impl Display for OpenIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other(msg) => write!(f, "Other error message: {}", msg),
            Self::MissingEnv(name) => write!(f, "Missing environment variable: {}", name),
            Self::BadCsrfToken => write!(f, "Bad CSRF token"), // don't disclose sensitive info
            Self::BadIssuer(url) => write!(f, "Bad issuer URL: {}", url)
        }
    }
}

/// authorization URL & associated security parameters
struct OpenIdUrl {
    url: Url,
    nonce: Nonce,
    state: CsrfToken
}

/// captures parameters passed by openid provider to our callback
#[derive(Debug, Deserialize)]
struct AuthenticationCallbackParameters {
    code: String,
    state: String
}

/// contains unvalidated parameters and the security parameters that should be
/// used to authenticated them
struct AuthenticationRequest {
    params: AuthenticationCallbackParameters,
    old_nonce: String,
    old_state: String
}
