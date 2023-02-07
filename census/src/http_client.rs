use reqwest::{
    Result,
    Response
};

pub struct HttpClient {
    root: String
}

impl HttpClient {
    pub fn new(root: &str) -> Self {
        Self {
            root: String::from(root)
        }
    }

    pub async fn get(&self, endpoint: &str) -> Result<Response> {
        reqwest::get(self.root.to_string() + endpoint)
            .await
    }
}