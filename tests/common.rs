use probe_schedule_client::upstream_api::probe_schedule::PSClient;
use probe_schedule_client::upstream_api::{ApiError, RestLoginBody, RestLoginResponse, Upstream};

pub enum KnownUpstream {
    Mock(MockUpstream),
    Ps(PSClient),
}

impl Upstream for KnownUpstream {
    async fn login(&self, body: RestLoginBody<'_>) -> tonic::Result<RestLoginResponse, ApiError> {
        match self {
            KnownUpstream::Mock(u) => u.login(body).await,
            KnownUpstream::Ps(u) => u.login(body).await,
        }
    }
}

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
