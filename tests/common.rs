use probe_schedule_client::sensor_data::{SensorDataRequest, SensorDataResponse};
use probe_schedule_client::upstream_api::{ApiError, Upstream};
use std::sync::RwLock;

#[derive(Debug, PartialEq)]
pub enum Token {
    Valid,
    Invalid,
}

pub struct MockUpstream {
    pub token: RwLock<Option<Token>>,
}

impl Upstream for MockUpstream {
    async fn login(&self) -> Result<(), ApiError> {
        *self.token.write().unwrap() = Some(Token::Valid);
        Ok(())
    }

    async fn sensor_data(
        &self,
        _body: SensorDataRequest,
    ) -> tonic::Result<SensorDataResponse, ApiError> {
        self.login().await?;
        Err(ApiError::Upstream {
            status: reqwest::StatusCode::NOT_IMPLEMENTED,
            body: "not implemented".to_string(),
        })
    }
}
