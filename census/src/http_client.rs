use common::user::User;
use reqwest::{
    Result,
    Response
};

pub struct HttpClient {
    bearer: Option<String>,
    root: String
}

impl HttpClient {
    pub fn new(root: &str) -> Self {
        Self {
            bearer: None,
            root: String::from(root)
        }
    }

    pub async fn authenticate_as(&mut self, user: &User) -> Result<()> {
        let result: String = reqwest::Client::new()
            .post(self.root.to_string() + "/jwt")
            .json(user)
            .send()
            .await?
            .text()
            .await?;
        self.bearer = Some(result);

        Ok(())
    }

    pub async fn get(&self, endpoint: &str) -> Result<Response> {
        let mut builder = reqwest::Client::new()
            .get(self.root.to_string() + endpoint);
        
        if let Some(ref token) = self.bearer {
            builder = builder.bearer_auth(token.to_owned());
        }

        let result = builder.send().await;
        println!("HTTP client received {:#?}", result);

        result
    }
}