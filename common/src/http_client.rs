// sets up authenticated & authorized client to consume APIs

use crate::user::User;
use reqwest::{
    Result,
    Response
};

/// basic HTTP client wrapper
pub struct HttpClient {

    /// root URL to make requests to
    root_url: String,

    /// bearer token retrieved from API
    bearer: Option<String>
}

impl HttpClient {

    /// Creates an unauthenticated client.
    /// root_url should not end with a '/'
    pub fn new(root_url: &str) -> Self {
        Self { 
            root_url: String::from(root_url), 
            bearer: None 
        }
    }

    /// Attempts to authenticate as the given user with the API.
    /// Ideally, this would use a more secure authentication scheme, but this is
    /// merely a demonstration of the techniques such a secure system could use,
    /// just as part of a more robust security system than this.
    pub async fn authenticate_as(&mut self, user: &User) -> Result<String> {
        let token: String = reqwest::Client::new()
            .post(self.root_url.to_owned() + "/jwt")
            .json(user)
            .send()
            .await?
            .text()
            .await?;
        self.bearer = Some(token.to_owned());

        Ok(token)
    }

    /// Makes a GET request to the given endpoint, which should begin with '/'.
    /// If authenticated, attached the Bearer header.
    pub async fn get(&self, endpoint: &str) -> Result<Response> {
        let mut builder = reqwest::Client::new()
            .get(self.root_url.to_owned() + endpoint);

        if let Some(ref token) = self.bearer {
            builder = builder.bearer_auth(token.to_owned());
        }

        let result = builder.send().await?;
        println!("HTTP client received {:#?}", result);

        Ok(result)
    }
}