use crate::sensor_data::{SensorDataRequest, SensorDataResponse};
use std::future::Future;
use super::error::ApiError;
use super::token::Token;

pub trait Transport: Send + Sync {
    fn login(&self) -> impl Future<Output = Result<Token, ApiError>> + Send;

    fn sensor_data(
        &self,
        token: &Token,
        body: &SensorDataRequest,
    ) -> impl Future<Output = Result<SensorDataResponse, ApiError>> + Send;
}

