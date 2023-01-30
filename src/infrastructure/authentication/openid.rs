// OpenID defers the authentication of users to a third-party service.

use std::{env, fmt::Display};

use openidconnect::{core::{CoreProviderMetadata, CoreClient, CoreResponseType}, IssuerUrl, reqwest::async_http_client, ClientId, RedirectUrl, CsrfToken, Nonce, AuthenticationFlow, Scope, ClientSecret};
use reqwest::Url;
use tokio::{task::JoinHandle, net::TcpListener, io::{BufReader, AsyncBufReadExt}};

static REDIRECT_URI: &str = "localhost:5000";

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
    MissingEnv(String)
}

impl OpenIdError {
    fn missing_env(name: &str) -> Self {
        Self::MissingEnv(String::from(name))
    }
}

impl Display for OpenIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented => write!(f, "OpenID not implemented"),
            Self::MissingEnv(name) => write!(f, "Missing environment variable: {}", name)
        }
    }
}

pub async fn setup_openid() -> Result<JoinHandle<()>, OpenIdError> {
    let handle = spawn_openid_callback_listener();
    let options = OpenIdOptions::from_env()?;
    do_openid(options)
        .await
        .map(|_| handle)
}

/// once the user authenticates with OpenID, our app needs to listen for a
/// message from the OpenID server, as it will give us an authorization code
/// we can use to obtain information granted in the scopes
fn spawn_openid_callback_listener() -> JoinHandle<()> {

    // spawn a new thread so it doesn't block the program
    tokio::spawn(async {

        let tcp_listener = TcpListener::bind(REDIRECT_URI)
            .await
            .expect(&format!("Should be able to bind to {}", REDIRECT_URI));
        
        // do forever
        loop {

            // listen for a response connection from the OpenID server
            let (mut conn, _socket) = tcp_listener.accept()
                .await
                .expect("Should accept TCP connections");
            
            // read the response
            let mut reader = BufReader::new(&mut conn);
            let mut line = String::new();
            reader.read_line(&mut line)
                .await
                .expect("Should be able to read line");
            println!("Line: {}", line);
            // line is GET {URL} HTTP/1.1

            // extract auth code
            let url = line.split_whitespace()
                .nth(1)
                .expect("line should contain URL after first whitespace");
            
            let parsed_url = Url::parse(&format!("http://localhost:5000{}", url))
                .expect("URL should be valid");

            let auth_code = parsed_url
                .query_pairs()
                .find(|pair| pair.0 == "code")
                .expect("Should contain query parameter 'code'")
                .1 // value
                .into_owned();
            
            let state = parsed_url
                .query_pairs()
                .find(|pair| pair.0 == "state")
                .expect("should contain query parameter 'state'")
                .1
                .into_owned();

            println!("Auth code: {}", auth_code);
            println!("State: {}", state);

            // todo https://github.com/ramosbugs/openidconnect-rs/blob/main/examples/google.rs
            // on line 184
        }
    })
}

async fn do_openid(options: OpenIdOptions) -> Result<(), OpenIdError> {
    let issuer_url = IssuerUrl::new(options.url)
        .expect("Expected a valid issuer URL");

    let provider_document = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
        .await
        .expect("Expected issuer to provide a provider document");
    //println!("Provider document: {:#?}", provider_document);

    let client = CoreClient::from_provider_metadata(
            provider_document,
            ClientId::new(options.client_id),
            options.client_secret.map(|secret| ClientSecret::new(secret))
        )
        .set_redirect_uri(
            RedirectUrl::new(String::from("http://") + REDIRECT_URI).expect("Expected valid URL")
        );
    
    // generate an authorization URL requesting the details we want
    let (authorization_url, _csrf_state, _nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode, 
            CsrfToken::new_random, 
            Nonce::new_random
        )
        .add_scope(Scope::new(String::from("email")))
        .add_scope(Scope::new(String::from("profile")))
        .url();
    
    println!("Authorization URL: {}", authorization_url);

    Ok(())
}

