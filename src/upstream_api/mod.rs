use crate::sensor_data::sensor_data_service_server::SensorDataService;
use crate::sensor_data::{SensorDataRequest, SensorDataResponse};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::RwLock;
use tonic::{Request, Response, Result, Status};
include!("upstream_api_error.rs");

pub mod probe_schedule;

#[derive(Clone, PartialEq, Eq)]
pub struct Token(std::sync::Arc<str>);

impl Token {
    pub fn new(s: String) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// keep secrets out of logs
impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Token(***)")
    }
}

pub trait Transport: Send + Sync {
    fn login(&self) -> impl Future<Output = Result<Token, ApiError>> + Send;

    fn sensor_data(
        &self,
        token: &Token,
        body: SensorDataRequest,
    ) -> impl Future<Output = Result<SensorDataResponse, ApiError>> + Send;
}

pub struct Authed<T: Transport + 'static> {
    transport: &'static T,
    pub token: RwLock<Option<Token>>,
    refresh_gate: tokio::sync::Mutex<()>,
}

impl<T: Transport + 'static> Authed<T> {
    pub fn new(t: &'static T) -> Self {
        Authed {
            transport: t,
            token: RwLock::default(),
            refresh_gate: tokio::sync::Mutex::default(),
        }
    }

    async fn ensure_token(&self) -> Token {
        match self.token.read().unwrap().clone() {
            Some(t) => t,
            None => Token::new(String::from("")),
        }
    }

    async fn refresh(&self, stale_token: &Token) -> Result<Token, ApiError> {
        self.refresh_gate.lock();
        let check_token = self.ensure_token().await;
        if &check_token != stale_token {
            return Ok(check_token);
        }
        let new_token = self.transport.login().await?;
        *self.token.write().unwrap() = Some(new_token.clone());
        Ok(new_token)
    }
}

#[tonic::async_trait]
impl<T: Transport + 'static> SensorDataService for Authed<T> {
    async fn sensor_data(
        &self,
        req: Request<SensorDataRequest>,
    ) -> Result<Response<SensorDataResponse>, Status> {
        let token = self.ensure_token().await;
        match self
            .transport
            .sensor_data(&token, req.into_inner().clone())
            .await
        {
            Err(ApiError::Upstream { status, body })
                if status == StatusCode::BAD_REQUEST
                    || status == StatusCode::UNPROCESSABLE_ENTITY
                    || status == StatusCode::UNAUTHORIZED =>
            {
                let token = self.refresh(&token).await?;
                self.transport.sensor_data(&token, body).await
            }
            other => other,
        }
    }
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
