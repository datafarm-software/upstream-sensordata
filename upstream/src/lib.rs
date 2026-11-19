use api::Query;
use api::error::ApiError;
use api::token::Token;
use api::transport::Transport;
use reqwest::StatusCode;
use sea_orm::{DatabaseConnection, DbErr};
use sensor_data::sensor_data_service_server::SensorDataService;
use sensor_data::{SensorDataRequest, SensorDataResponse};
use std::sync::RwLock;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

pub mod api;
pub mod sensor_data {
    include!("../generated/rust/sensor_data/v1/sensor_data.v1.rs");
}

pub struct Authed<T: Transport> {
    refresh_gate: Mutex<()>,
    pub token: RwLock<Option<Token>>,
    transport: T,
    db: DatabaseConnection,
}

impl<T: Transport> Authed<T> {
    pub async fn new(transport: T, db: DatabaseConnection) -> Self {
        Self {
            transport,
            db,
            token: RwLock::new(None),
            refresh_gate: Mutex::new(()),
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
        let di = match Query::find_device_info(&self.db, &body.device_id).await {
            Ok(Some(di)) => di,
            Ok(None) => return Err(Status::not_found("device not found")),
            Err(e) => match e {
                DbErr::RecordNotFound(msg) => return Err(Status::not_found(msg)),
                _ => {
                    println!("unexpected sql error: {e}");
                    return Err(Status::internal("unexpected sql error"));
                }
            },
        };
        let res = match self.transport.sensor_data(&token, &di).await {
            Err(ApiError::Upstream { status, .. })
                if status == StatusCode::BAD_REQUEST
                    || status == StatusCode::UNPROCESSABLE_ENTITY
                    || status == StatusCode::UNAUTHORIZED =>
            {
                let token = self.refresh(&token).await?;
                self.transport.sensor_data(&token, &di).await
            }
            other => other,
        };
        res.map(Response::new).map_err(Status::from)
    }
}
