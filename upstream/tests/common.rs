use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use upstream::api::{error::ApiError, token::Token, transport::Transport};
use upstream::sensor_data::{SensorData, SensorDataRequest, SensorDataResponse};

pub struct MockUpstream {
    pub mock_data: Arc<SensorData>,
    pub login_called: Arc<AtomicUsize>,
}

pub const VALID_TOKEN: &str = "Valid Token";
pub const INVALID_TOKEN: &str = "Invalid Token";

impl Transport for MockUpstream {
    async fn login(&self) -> Result<Token, ApiError> {
        self.login_called.fetch_add(1, Ordering::SeqCst);
        tokio::task::yield_now().await;
        Ok(Token::new(String::from(VALID_TOKEN)))
    }

    async fn sensor_data(
        &self,
        token: &Token,
        _body: &SensorDataRequest,
    ) -> tonic::Result<SensorDataResponse, ApiError> {
        if *token == Token::new(String::from(VALID_TOKEN)) {
            Ok(SensorDataResponse {
                response: vec![Arc::unwrap_or_clone(self.mock_data.clone())],
            })
        } else {
            Err(ApiError::Upstream {
                status: reqwest::StatusCode::UNAUTHORIZED,
                body: "unauthorized".to_string(),
            })
        }
    }
}
