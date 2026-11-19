use super::error::ApiError;
use super::token::Token;
use super::transport::Transport;
use chrono::{DateTime, Utc};
use entity::device_info::Model as DeviceInfo;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use serde_tuple::Deserialize_tuple;
use std::collections::HashMap;
use std::time::Duration;
use tonic::Result;

#[derive(Serialize)]
pub struct RestLoginBody<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Deserialize)]
pub struct RestLoginResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Opts {
    pub base_url: String,
    pub username: String,
    pub password: String,
}

#[derive(Default)]
pub struct PSClient {
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

#[derive(Deserialize)]
pub struct PSResponse {
    pub success: bool,
    pub data: Data,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub sensors: HashMap<String, Sensor>,
}

#[derive(Debug, Deserialize)]
pub struct Sensor {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub readings: Vec<Reading>,
}

#[derive(Debug, Deserialize_tuple)]
pub struct Reading {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub time: DateTime<Utc>,
    pub value: f64,
}

impl Transport for PSClient {
    async fn login(&self) -> Result<Token, ApiError> {
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
        Ok(Token::new(reply.token))
    }

    async fn sensor_data(
        &self,
        token: &Token,
        body: &DeviceInfo,
    ) -> Result<crate::sensor_data::SensorDataResponse, ApiError> {
        let url = format!("{0}/{1}/data", self.base_url, body.block_id);
        let resp: PSResponse = self
            .http
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {0}", token.as_str()))
            .send()
            .await?
            .json::<PSResponse>()
            .await?;
    }
}
