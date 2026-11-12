use probe_schedule_client::sensor_data::{SensorDataRequest, SensorDataResponse};
use probe_schedule_client::upstream_api::{ApiError, RestLoginBody, RestLoginResponse, Upstream};

#[derive(Debug, PartialEq)]
pub enum Token {
    Valid,
    Invalid,
}

pub struct MockUpstream {
    pub token: Token,
    pub return_code: tonic::Code,
}

impl Upstream for MockUpstream {
    async fn login(&self, _: RestLoginBody<'_>) -> tonic::Result<RestLoginResponse, ApiError> {
        Ok(RestLoginResponse {
            token: "valid-token".to_string(),
        })
    }

    async fn sensor_data(
        &self,
        body: SensorDataRequest,
    ) -> tonic::Result<SensorDataResponse, ApiError> {
        todo!()
    }
}
