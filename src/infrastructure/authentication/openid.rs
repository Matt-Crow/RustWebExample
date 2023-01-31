// OpenID defers the authentication of users to a third-party service.
// derived from 
// https://github.com/ramosbugs/openidconnect-rs/blob/main/examples/google.rs

use std::{env, fmt::{Display, Debug}};

use actix_web::{web::{ServiceConfig, get, self}, Responder, HttpResponse, error};
use openidconnect::{core::{CoreProviderMetadata, CoreClient, CoreResponseType}, IssuerUrl, reqwest::async_http_client, ClientId, RedirectUrl, CsrfToken, Nonce, AuthenticationFlow, Scope, ClientSecret, AuthorizationCode};
use reqwest::Url;
use serde::Deserialize;

/// Open ID configuration options. Stored as environment variables
#[derive(Debug)]
pub struct OpenIdOptions {
    url: String,
    client_id: String,
    client_secret: Option<String>
}

impl OpenIdOptions {
    pub fn new(url: &str, client_id: &str) -> Self {
        Self {
            url: String::from(url),
            client_id: String::from(client_id),
            client_secret: None
        }
    }

    pub fn with_secret(&self, client_secret: &str) -> Self {
        Self {
            url: self.url.to_owned(),
            client_id: self.client_id.to_owned(),
            client_secret: Some(String::from(client_secret))
        }
    }

    pub fn from_env() -> Result<Self, OpenIdError> {
        let url = env::var("OPENID_URL")
            .map_err(|_| OpenIdError::missing_env("OPENID_URL"))?;
        let id = env::var("OPENID_CLIENT_ID")
            .map_err(|_| OpenIdError::missing_env("OPENID_CLIENT_ID"))?;
        let secret = env::var("OPENID_CLIENT_SECRET")
            .ok(); // can be None
        
        let n = match secret {
            Some(secret) => Self::new(&url, &id).with_secret(&secret),
            None => Self::new(&url, &id)
        };

        Ok(n)
    }
}

#[derive(Debug)]
pub enum OpenIdError {
    NotImplemented,
    Other(String),
    Debug(String),
    MissingEnv(String),
    BadIssuer(String)
}

impl OpenIdError {
    fn other(obj: impl ToString) -> Self {
        Self::Other(obj.to_string())
    }

    fn debug(obj: impl Debug) -> Self {
        Self::Debug(format!("OpenIdError::Debug({:#?})", obj))
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
            Self::Debug(msg) => write!(f, "{}", msg),
            Self::MissingEnv(name) => write!(f, "Missing environment variable: {}", name),
            Self::BadIssuer(url) => write!(f, "Bad issuer URL: {}", url)
        }
    }
}

#[derive(Debug)]
pub struct OpenIdService {
    client: CoreClient,
    token: Option<CsrfToken> // temp
}

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
                options.client_secret.map(|secret| ClientSecret::new(secret))
            )
            .set_redirect_uri(
                RedirectUrl::new(String::from("http://localhost:8080/openid")).expect("Expected valid URL")
            );

        Ok(Self {
            client,
            token: None
        })
    }

    pub async fn from_env() -> Result<Self, OpenIdError> {
        Self::new(OpenIdOptions::from_env()?).await
    }

    pub async fn generate_auth_url(&mut self) -> Url {
        // generate an authorization URL requesting the details we want
        let (authorization_url, csrf_state, _nonce) = self.client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode, 
                CsrfToken::new_random, 
                Nonce::new_random
            )
            .add_scope(Scope::new(String::from("email")))
            .add_scope(Scope::new(String::from("profile")))
            .url();
        self.token = Some(csrf_state);

        println!("Authorization URL: {}", authorization_url);

        authorization_url
    }

    /// called after a user authenticates with the OpenID provider
    async fn handle_callback(
        &self, 
        params: AuthenticationCallbackParameters
    ) -> Result<(), OpenIdError> {
        
        // validate CSRF token
        // how do I handle having multiple CSRF tokens?
        // use them as a PK somewhere?
        let expected = self.token.clone()
            .ok_or_else(|| OpenIdError::NotImplemented)?
            .secret()
            .to_string();
        
        if expected != params.state {
            panic!("How do I properly validate CSRF?");
        }
            
        self.exchange(params.code.to_owned())
            .await?;

        Ok(())
    }

    pub async fn exchange(&self, code: String) -> Result<(), OpenIdError> {
        let token_response = self.client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .map_err(OpenIdError::debug)?;
        // fails here: deserialization issues?
        /*
            Error {
                path: Path {
                    segments: [],
                },
                original: Error("Failed to parse payload JSON: Error(\"data did not match any variant of untagged enum Timestamp\", line: 1, column: 535)", line: 1, column: 1739),
            }
        
            https://docs.rs/serde_path_to_error/latest/serde_path_to_error/
            Why does path list no segments? is it assuming the root is a Timestamp?

            Here's what the payload looks like:
            {
                "access_token": "...",
                "id_token": "...",
                "scope": "openid profile email",
                "expires_in": 86400,
                "token_type": "Bearer"
            }

            "untagged" enums just mean the JSON doesn't explicitly state what variant of the enum it is
            https://serde.rs/enum-representations.html

            Timestamp looks like this
                #[derive(Debug, Deserialize, Serialize)]
                #[serde(untagged)]
                pub(crate) enum Timestamp {
                    Seconds(serde_json::Number),
                    #[cfg(feature = "accept-rfc3339-timestamps")]
                    Rfc3339(String),
                }
            https://github.com/ramosbugs/openidconnect-rs/blob/4ba900b1e9c6e552bd29fdf891e0f0f8b690b35d/src/types.rs

            serde_json::Number looks like this
                #[derive(Clone, PartialEq, Eq, Hash)]
                pub struct Number {
                    n: N,
                }

                #[cfg(not(feature = "arbitrary_precision"))]
                #[derive(Copy, Clone)]
                enum N {
                    PosInt(u64),
                    /// Always less than zero.
                    NegInt(i64),
                    /// Always finite.
                    Float(f64),
                }
            https://docs.rs/serde_json/latest/src/serde_json/number.rs.html#22-24
        */
        
        println!("token response: {:#?}", token_response);

        Ok(())
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
    println!("data: {:#?}", openid_response);

    let f = service.handle_callback(openid_response.0)
        .await
        .map(|nothing_yet| HttpResponse::Ok().body("todo put bearer header in body"))
        .map_err(|err| error::ErrorBadRequest(err));
    f
}

