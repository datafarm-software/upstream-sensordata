use serde::{Deserialize, Serialize};
use tonic::{Request, Response, Result, Status};
include!("upstream_api_error.rs");

pub trait Upstream: Send + Sync {
    async fn login(&self, body: RestLoginBody) -> Result<RestLoginResponse, ApiError>;
}

#[derive(Serialize)]
pub struct RestLoginBody<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Deserialize)]
pub struct RestLoginResponse {
    pub token: String,
}
