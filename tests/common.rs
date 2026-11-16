use std::sync::atomic::{AtomicUsize, Ordering};
use upstream_sensordata::sensor_data::{SensorData, SensorDataRequest, SensorDataResponse};
use upstream_sensordata::upstream_api::{ApiError, Token, Transport};

pub struct MockUpstream<'a> {
    pub mock_data: &'a SensorData,
    pub login_called: AtomicUsize,
}

pub const VALID_TOKEN: &str = "Valid Token";
pub const INVALID_TOKEN: &str = "Invalid Token";

impl<'a> MockUpstream<'a> {
    pub fn login_count(&self) -> usize {
        self.login_called.load(Ordering::SeqCst)
    }
}

impl<'a> Transport for MockUpstream<'a> {
    async fn login(&self) -> Result<Token, ApiError> {
        self.login_called.fetch_add(1, Ordering::SeqCst);
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
