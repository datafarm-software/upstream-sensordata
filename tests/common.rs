use probe_schedule_client::upstream_api::{ApiError, RestLoginBody, RestLoginResponse, Upstream};

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
}
