use crate::api::error::ApiError;
use crate::api::token::Token;
use crate::api::transport::Transport;
use crate::sensor_data::sensor_data_service_server::SensorDataService;
use crate::sensor_data::{SensorDataRequest, SensorDataResponse};
use reqwest::StatusCode;
use std::sync::RwLock;
use tonic::{Request, Response, Status};

pub mod api;
pub mod sensor_data {
    include!("../generated/rust/sensor_data/v1/sensor_data.v1.rs");
}

pub struct Authed<T> {
    refresh_gate: tokio::sync::Mutex<()>,
    pub token: RwLock<Option<Token>>,
    transport: T,
}

impl<T: Transport> Authed<T> {
    pub fn new(t: T) -> Self {
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
        let _guard = self.refresh_gate.lock().await;
        let check_token = self.ensure_token().await;
        if &check_token != stale_token {
            // Another thread refreshed while this one waited.
            // Take the updated token.
            return Ok(check_token);
        }
        let new_token = self.transport.login().await?;
        *self.token.write().unwrap() = Some(new_token.clone());
        Ok(new_token)
    }
}

#[tonic::async_trait]
impl<T> SensorDataService for Authed<T>
where
    T: Transport + Send + Sync + 'static,
{
    async fn sensor_data(
        &self,
        req: Request<SensorDataRequest>,
    ) -> Result<Response<SensorDataResponse>, Status> {
        let token = self.ensure_token().await;
        let body = req.into_inner();
        let res = match self.transport.sensor_data(&token, &body).await {
            Err(ApiError::Upstream { status, .. })
                if status == StatusCode::BAD_REQUEST
                    || status == StatusCode::UNPROCESSABLE_ENTITY
                    || status == StatusCode::UNAUTHORIZED =>
            {
                let token = self.refresh(&token).await?;
                self.transport.sensor_data(&token, &body).await
            }
            other => other,
        };
        res.map(Response::new).map_err(Status::from)
    }
}
