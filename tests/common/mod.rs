use upstream_api::Upstream;

struct MockUpstream {
    return_code: reqwest::StatusCode
}

impl Upstream for MockUpstream {}

pub fn setup() {}
