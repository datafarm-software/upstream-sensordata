use crate::upstream_api::Upstream;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tonic::{Request, Response, Result, Status};

pub mod psclient_grpc {
    include!("../gen/rust/psclient/v1/psclient.v1.rs");
    include!("../gen/rust/psclient/v1/psclient.v1.tonic.rs");
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Opts {
    base_url: String,
    username: String,
    password: String,
}

#[derive(Default)]
struct PSClient {
    base_url: String,
    username: String,
    password: String,
    http: reqwest::Client,
}

impl PSClient {
    fn new(opts: Opts) -> Result<Self, reqwest::Error> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;
        Ok(Self {
            base_url: opts.base_url,
            username: opts.username,
            password: opts.password,
            http,
        })
    }

    async fn do_login(&self, req: &LoginRequest) -> Result<LoginResponse, ApiError> {
        let resp = self
            .http
            .post(format!("{}/token", self.base_url))
            .json(&RestLoginBody {
                username: &req.username,
                password: &req.password,
            })
            .send()
            .await?;

        let resp = check_status(resp).await?;
        let reply: RestLoginResponse = resp.json().await?;
        Ok(LoginResponse { token: reply.token })
    }
}

async fn check_status(resp: reqwest::Response) -> Result<reqwest::Response, ApiError> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let body = resp.text().await.unwrap_or_default();
    Err(ApiError::Upstream { status, body })
}

impl Upstream for PSClient {}
