use crate::sensor_data::{SensorDataRequest, SensorDataResponse};
use serde::{Deserialize, Serialize};
use std::future::Future;
use tonic::{Result, Status};
include!("upstream_api_error.rs");

pub mod probe_schedule;

pub trait Upstream: Send + Sync {
    fn login(&self) -> impl Future<Output = Result<(), ApiError>> + Send;

    fn sensor_data(
        &self,
        body: SensorDataRequest,
    ) -> impl Future<Output = Result<SensorDataResponse, ApiError>> + Send;
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
