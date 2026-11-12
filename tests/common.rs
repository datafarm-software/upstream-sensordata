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
    async fn login(&self) -> Option<ApiError> {
        self.token = Token::Valid;
    }

    async fn sensor_data(
        &self,
        _body: SensorDataRequest,
    ) -> tonic::Result<SensorDataResponse, ApiError> {
        self.login();
    }
}
