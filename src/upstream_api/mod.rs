use crate::sensor_data::{SensorDataRequest, SensorDataResponse};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::RwLock;
use tonic::{Result, Status};
include!("upstream_api_error.rs");

pub mod probe_schedule;

pub type Token = &'static str;

pub trait Transport: Send + Sync {
    fn login(&self) -> impl Future<Output = Result<Token, ApiError>> + Send;

    fn sensor_data(
        &self,
        token: &Token,
        body: SensorDataRequest,
    ) -> impl Future<Output = Result<SensorDataResponse, ApiError>> + Send;
}

pub struct Authed<T: Transport> {
    transport: T,
    token: RwLock<Option<Token>>,
    refresh_gate: tokio::sync::Mutex<()>,
}

impl<T: Transport> Authed<T> {}

#[derive(Serialize)]
pub struct RestLoginBody<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Deserialize)]
pub struct RestLoginResponse {
    pub token: String,
}
