#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("upstream request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("upstream returned {status}")]
    Upstream { status: StatusCode, body: String },
}

impl From<ApiError> for Status {
    fn from(err: ApiError) -> Self {
        match err {
            ApiError::Http(e) if e.is_timeout() => Status::deadline_exceeded("upstream timed out"),
            ApiError::Http(e) if e.is_connect() => Status::unavailable("could not reach upstream"),
            ApiError::Http(e) if e.is_decode() => {
                Status::internal("unexpected response from upstream")
            }
            ApiError::Http(_) => Status::internal("upstream call failed"),
            ApiError::Upstream { status, .. } => match status.as_u16() {
                400 | 422 => Status::invalid_argument("upstream rejeceted request"),
                401 => Status::unauthenticated("invalid token or credentials"),
                403 => Status::permission_denied("upstream denied access"),
                429 => Status::resource_exhausted("upstream rate limit reached"),
                500..=599 => Status::unavailable("upstream error"),
                _ => Status::internal("unexpected upstream status"),
            },
        }
    }
}
