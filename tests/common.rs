use probe_schedule_client::sensor_data::{SensorDataRequest, SensorDataResponse};
use probe_schedule_client::upstream_api::{ApiError, Token, Transport};
use std::sync::RwLock;

pub struct MockUpstream {}

pub const VALID_TOKEN: Token = "Valid Token";
pub const INVALID_TOKEN: Token = "Invalid Token";

impl Transport for MockUpstream {
    async fn login(&self) -> Result<Token, ApiError> {
        Ok(Token::from("Valid Token"))
    }

    async fn sensor_data(
        &self,
        token: &Token,
        _body: SensorDataRequest,
    ) -> tonic::Result<SensorDataResponse, ApiError> {
        Err(ApiError::Upstream {
            status: reqwest::StatusCode::NOT_IMPLEMENTED,
            body: "not implemented".to_string(),
        })
    }
}
