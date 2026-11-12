use crate::error::ApiError;
use crate::psclient::psclient_grpc::ps_client_service_server::PsClientService;
use crate::psclient::psclient_grpc::{LoginRequest, LoginResponse};
use serde::{Deserialize, Serialize};
use serde_json::{self};
use serde_yaml::{self};
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
            .json(&LoginBody {
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

#[derive(Serialize)]
struct LoginBody<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Deserialize)]
struct RestLoginResponse {
    token: String,
}

#[tonic::async_trait]
impl PsClientService for PSClient {
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let resp = self.do_login(&request.into_inner()).await?;
        Ok(Response::new(resp))
    }
}
