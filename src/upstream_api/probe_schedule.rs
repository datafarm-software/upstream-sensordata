use crate::upstream_api::{ApiError, RestLoginBody, RestLoginResponse, Upstream};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use std::time::Duration;
use tonic::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct Opts {
    pub base_url: String,
    pub username: String,
    pub password: String,
}

#[derive(Default)]
pub struct PSClient {
    token: RwLock<Option<String>>,
    base_url: String,
    username: String,
    password: String,
    http: reqwest::Client,
}

impl PSClient {
    pub fn new(opts: Opts) -> Result<Self, reqwest::Error> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;
        Ok(Self {
            token: RwLock::default(),
            base_url: opts.base_url,
            username: opts.username,
            password: opts.password,
            http,
        })
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

impl Upstream for PSClient {
    async fn login(&self) -> Result<(), ApiError> {
        let resp = self
            .http
            .post(format!("{}/token", self.base_url))
            .json(&RestLoginBody {
                username: &self.username,
                password: &self.password,
            })
            .send()
            .await?;

        let resp = check_status(resp).await?;
        let reply: RestLoginResponse = resp.json().await?;
        *self.token.write().unwrap() = Some(reply.token);
        Ok(())
    }

    async fn sensor_data(
        &self,
        _body: crate::sensor_data::SensorDataRequest,
    ) -> Result<crate::sensor_data::SensorDataResponse, ApiError> {
        todo!()
    }
}
