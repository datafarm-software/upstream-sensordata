mod common;
use common::MockUpstream;
use probe_schedule_client::sensor_data::SensorDataRequest;
use probe_schedule_client::upstream_api::{Token, Transport};

struct Case {
    transport: MockUpstream,
    input: SensorDataRequest,
    name: &'static str,
    want_token: Token,
}

#[tokio::test]
async fn test_lazy_login() {
    let cases = vec![Case {
        name: "login on first request received",
        transport: MockUpstream {},
        want_token: common::VALID_TOKEN,
        input: SensorDataRequest {
            device_id: "device1".to_string(),
        },
    }];
    for tc in cases {
        println!("case: {}", tc.name);
        let _got = tc.upstream.sensor_data(tc.input).await;
        assert_eq!(*tc.upstream.token.read().unwrap(), Some(tc.want_token));
    }
}
