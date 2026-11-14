use upstream_sensordata::sensor_data::{SensorDataRequest, SensorDataResponse};
use upstream_sensordata::upstream_api::{ApiError, Token, Transport};
use std::sync::RwLock;

pub struct MockUpstream {}

pub const VALID_TOKEN: &str = "Valid Token";
pub const INVALID_TOKEN: &str = "Invalid Token";

impl Transport for MockUpstream {
    async fn login(&self) -> Result<Token, ApiError> {
        Ok(Token::new(String::from(VALID_TOKEN)))
    }

    async fn sensor_data(
        &self,
        token: &Token,
        _body: SensorDataRequest,
    ) -> tonic::Result<SensorDataResponse, ApiError> {
        Err(ApiError::Upstream {
            status: reqwest::StatusCode::UNAUTHORIZED,
            body: "unauthorized".to_string(),
        })
    }
}
