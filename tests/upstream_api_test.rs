mod common;
use common::{MockUpstream, Token};
use probe_schedule_client::sensor_data::SensorDataRequest;
use probe_schedule_client::upstream_api::Upstream;
use std::sync::RwLock;

struct Case {
    upstream: MockUpstream,
    input: SensorDataRequest,
    _name: &'static str,
    want_token: Token,
}

#[tokio::test]
async fn test_lazy_login() {
    let cases = vec![Case {
        _name: "login on first request received",
        upstream: MockUpstream {
            token: RwLock::new(Some(Token::Invalid)),
        },
        want_token: Token::Valid,
        input: SensorDataRequest {
            device_id: "device1".to_string(),
        },
    }];
    for tc in cases {
        let _got = tc.upstream.sensor_data(tc.input).await;
        assert_eq!(*tc.upstream.token.read().unwrap(), Some(tc.want_token));
    }
}
