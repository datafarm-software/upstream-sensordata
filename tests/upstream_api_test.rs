use upstream_sensordata::sensor_data::sensor_data_service_server::SensorDataService;
mod common;
use common::MockUpstream;
use upstream_sensordata::sensor_data::SensorDataRequest;
use upstream_sensordata::upstream_api;
use upstream_api::Token;

struct Case {
    transport: &'static MockUpstream,
    input: SensorDataRequest,
    name: &'static str,
    want_token: Token,
}

#[tokio::test(flavor = "current_thread")]
async fn test_lazy_login() {
    let cases = vec![Case {
        name: "login on first request received",
        transport: &MockUpstream {},
        want_token: Token::new(String::from(common::VALID_TOKEN)),
        input: SensorDataRequest {
            device_id: "device1".to_string(),
        },
    }];
    for tc in cases {
        println!("case: {}", tc.name);
        let authed = upstream_api::Authed::new(tc.transport);
        let _got = authed.sensor_data(tonic::Request::new(tc.input)).await;
        assert_eq!(authed.token.read().unwrap().clone(), Some(tc.want_token));
    }
}
