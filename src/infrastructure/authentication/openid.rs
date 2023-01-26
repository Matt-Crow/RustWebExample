// OpenID defers the authentication of users to a third-party service.

use openidconnect::{core::{CoreProviderMetadata, CoreClient, CoreResponseType}, IssuerUrl, reqwest::async_http_client, ClientId, RedirectUrl, CsrfToken, Nonce, AuthenticationFlow, Scope};
use tokio::{task::JoinHandle, net::TcpListener, io::{BufReader, AsyncBufReadExt}};

static REDIRECT_URI: &str = "localhost:5000";

#[derive(Debug)]
pub enum OpenIdError {
    _NotImplemented
}

/// once the user authenticates with OpenID, our app needs to listen for a
/// message from the OpenID server, as it will give us an authorization code
/// we can use to obtain information granted in the scopes
pub fn spawn_openid_callback_listener() -> JoinHandle<()> {

    // spawn a new thread so it doesn't block the program
    tokio::spawn(async {

        // todo need to register redirect URI
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

            // todo extract auth code
        }
    })
}

pub async fn do_openid() -> Result<(), OpenIdError> {
    let url = "https://samples.auth0.com/";
    let client_id = "kbyuFDidLLm280LIwVFiazOqjO3ty8KH";
    let redirect_uri = "https://openidconnect.net/callback";

    let issuer_url = IssuerUrl::new(String::from(url))
        .expect("Expected a valid issuer URL");

    let provider_document = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
        .await
        .expect("Expected issuer to provide a provider document");
    println!("Provider document: {:#?}", provider_document);

    let client = CoreClient::from_provider_metadata(
            provider_document,
            ClientId::new(String::from(client_id)),
            None // no client secret
        )
        .set_redirect_uri(
            RedirectUrl::new(String::from(redirect_uri)).expect("Expected valid URL")
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