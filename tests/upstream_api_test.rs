mod common;
use common::{MockUpstream, Token};
use probe_schedule_client::sensor_data::SensorDataRequest;
use probe_schedule_client::upstream_api::Upstream;
use std::sync::RwLock;
use tonic::Code;

struct Case {
    upstream: MockUpstream,
    input: SensorDataRequest,
    name: &'static str,
    want_token: Token,
}

#[tokio::test]
async fn test_lazy_login() {
    let cases = vec![Case {
        name: "login on first request received",
        upstream: MockUpstream {
            token: RwLock::new(Some(Token::Invalid)),
            return_code: Code::Unauthenticated,
        },
        want_token: Token::Valid,
        input: SensorDataRequest {
            device_id: "device1".to_string(),
        },
    }];
    for tc in cases {
        let _got = tc.upstream.sensor_data(tc.input);
        assert_eq!(*tc.upstream.token.read().unwrap(), Some(tc.want_token));
    }
}
