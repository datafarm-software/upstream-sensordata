use super::error::ApiError;
use super::token::Token;
use crate::sensor_data::SensorDataResponse;
use entity::device_info::Model;
use std::future::Future;

pub trait Transport: Send + Sync {
    fn login(&self) -> impl Future<Output = Result<Token, ApiError>> + Send;

    fn sensor_data(
        &self,
        token: &Token,
        body: &Model,
    ) -> impl Future<Output = Result<SensorDataResponse, ApiError>> + Send;
}
