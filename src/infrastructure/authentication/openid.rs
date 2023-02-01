// OpenID defers the authentication of users to a third-party service.
// derived from 
// https://github.com/ramosbugs/openidconnect-rs/blob/main/examples/google.rs
// https://openid.net/specs/openid-connect-basic-1_0.html

use std::{env, fmt::{Display, Debug}, error::Error};

use actix_web::{web::{ServiceConfig, get, self}, Responder, HttpResponse, error};
use openidconnect::{core::{CoreProviderMetadata, CoreClient, CoreResponseType, CoreAuthPrompt}, IssuerUrl, reqwest::async_http_client, ClientId, RedirectUrl, CsrfToken, Nonce, AuthenticationFlow, Scope, ClientSecret, AuthorizationCode};
use reqwest::Url;
use serde::{Deserialize, Serialize};

/// Open ID configuration options. Stored as environment variables
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
    NotImplemented,
    Other(String),
    MissingEnv(String),
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
            Self::NotImplemented => write!(f, "OpenID not implemented"),
            Self::Other(msg) => write!(f, "Other error message: {}", msg),
            Self::MissingEnv(name) => write!(f, "Missing environment variable: {}", name),
            Self::BadIssuer(url) => write!(f, "Bad issuer URL: {}", url)
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OpenIdUser {
    email: String,
    groups: Vec<String>
}

#[derive(Debug)]
pub struct OpenIdService {
    client: CoreClient,
    token: Option<CsrfToken>, // temp. needed because this will be verified later
    nonce: Option<Nonce>, // temp. needed because this will be verified later
}
// nonce & token are both generated with the auth URL, but are needed in the
// callback - what is the best way to save them for later?

impl OpenIdService {
    
    pub async fn new(options: OpenIdOptions) -> Result<Self, OpenIdError> {
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
            client,
            token: None,
            nonce: None
        })
    }

    pub async fn from_env() -> Result<Self, OpenIdError> {
        Self::new(OpenIdOptions::from_env()?).await
    }

    pub async fn generate_auth_url(&mut self) -> Url {
        // generate an authorization URL requesting the details we want
        let (authorization_url, csrf_state, nonce) = self.client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode, 
                CsrfToken::new_random, 
                Nonce::new_random
            )
            .add_scope(Scope::new(String::from("email")))
            .add_scope(Scope::new(String::from("profile")))
            .add_prompt(CoreAuthPrompt::Consent)
            .url();
        self.token = Some(csrf_state);
        self.nonce = Some(nonce);

        println!("Authorization URL: {}", authorization_url);

        authorization_url
    }

    /// called after a user authenticates with the OpenID provider
    async fn handle_callback(
        &self, 
        params: AuthenticationCallbackParameters
    ) -> Result<OpenIdUser, OpenIdError> {
        
        // validate CSRF token
        // how do I handle having multiple CSRF tokens?
        // use them as a PK somewhere?
        let expected = self.token.clone()
            .ok_or(OpenIdError::NotImplemented)?
            .secret()
            .to_string();
        
        if expected != params.state {
            panic!("How do I properly validate CSRF?");
        }
            
        let user = self.exchange_token_for_claims(params.code.to_owned())
            .await?;

        Ok(user)
    }

    /// once the user authorizes the OpenID provider, this method takes the
    /// authorization code, and exchanges it for a set of claims about the user,
    /// such as their email address and user groups
    async fn exchange_token_for_claims(&self, code: String) -> Result<OpenIdUser, OpenIdError> {
        let token_response = self.client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .map_err(OpenIdError::trace)?;
        
        //println!("token response: {:#?}", token_response);

        let claims = token_response
            .extra_fields()
            .id_token()
            .expect("should contain ID token")
            .claims(&self.client.id_token_verifier(), &self.nonce.clone().unwrap())
            .map_err(OpenIdError::trace)?;
        
        let email = claims.email()
            .expect("ID token should contain email")
            .to_string();

        Ok(OpenIdUser { 
            email, 
            groups: vec![String::from("todo")]
        })
    }
}

#[derive(Debug, Deserialize)]
struct AuthenticationCallbackParameters {
    code: String,
    state: String
}

/// sets up the open ID callback handler
pub fn configure_openid_routes(cfg: &mut ServiceConfig) {
    cfg.route("/openid", get().to(handle_auth_callback));
}

/// once the user authenticates with OpenID, our app needs to listen for a
/// message from the OpenID server, as it will give us an authorization code
/// we can use to obtain information granted in the scopes
async fn handle_auth_callback(
    service: web::Data<OpenIdService>,
    openid_response: web::Query<AuthenticationCallbackParameters>
) -> impl Responder {

    // todo: should this return access token for use as bearer? 
    service.handle_callback(openid_response.0)
        .await
        .map(|user| HttpResponse::Ok().json(user))
        .map_err(error::ErrorBadRequest)
}

