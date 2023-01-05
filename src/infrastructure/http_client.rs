use reqwest::{
    Result,
    Response
};

pub async fn get(url: &str) -> Result<Response> {
    reqwest::get(url).await
}