use std::collections::HashMap;
use std::sync::LazyLock;
use upstream_sensordata::sensor_data::sensor_data_service_server::SensorDataService;
mod common;
use common::MockUpstream;
use upstream_api::Token;
use upstream_sensordata::sensor_data::{SensorData, SensorDataRequest};
use upstream_sensordata::upstream_api;

struct Case {
    name: &'static str,
    want_token: Token,
    threads: usize,
}

static MOCK_DATA: LazyLock<SensorData> = LazyLock::new(|| SensorData {
    device_id: "device1".to_string(),
    timestamp: "2026-11-16T08:00:00Z".to_string(),
    sensor_data: HashMap::from([("temperature".to_string(), 23.00)]),
});

static MOCK_TRANSPORT: LazyLock<MockUpstream> = LazyLock::new(|| MockUpstream {
    mock_data: &MOCK_DATA,
});
static INPUT: LazyLock<SensorDataRequest> = LazyLock::new(|| SensorDataRequest {
    device_id: "device1".to_string(),
});

#[tokio::test(flavor = "current_thread")]
async fn test_lazy_login() {
    let cases = vec![Case {
        name: "login on first request received",
        want_token: Token::new(String::from(common::VALID_TOKEN)),
        threads: 1,
    }];
    for tc in cases {
        println!("case: {}", tc.name);
        let authed = upstream_api::Authed::new(&*MOCK_TRANSPORT);
        let futs = (0..tc.threads).map(|_| authed.sensor_data(tonic::Request::new(INPUT.clone())));
        let _results = futures::future::join_all(futs).await;

        assert_eq!(authed.token.read().unwrap().clone(), Some(tc.want_token));
    }
}
