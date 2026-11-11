use crate::psclient::psclient_grpc::ps_client_service_server::PsClientService;
use crate::psclient::psclient_grpc::{LoginRequest, LoginResponse};
use serde::{Deserialize, Serialize};
use serde_json::{self};
use serde_yaml::{self};
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
    fn new(opts: Opts) -> Self {
        let http = reqwest::Client::new();
        Self {
            base_url: opts.base_url,
            username: opts.username,
            password: opts.password,
            http,
        }
    }
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
        let url = format!("{}/token", self.base_url);
        let body = LoginBody {
            username: &self.username,
            password: &self.password,
        };
        let reply: RestLoginResponse = self.http.post(url).json(&body).send().await?.json().await?;
        return Ok();
    }
}
