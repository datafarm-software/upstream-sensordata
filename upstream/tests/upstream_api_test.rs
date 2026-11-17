use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::SystemTime;
use upstream_sensordata::sensor_data::sensor_data_service_server::SensorDataService;
mod common;
use common::MockUpstream;
use prost_types::Timestamp;
use std::sync::atomic::AtomicUsize;
use upstream_api::Token;
use upstream_sensordata::sensor_data::{SensorData, SensorDataRequest};
use upstream_sensordata::upstream_api;

struct LoginCase {
    name: &'static str,
    want_token: Token,
    threads: usize,
}

static MOCK_DATA: LazyLock<SensorData> = LazyLock::new(|| SensorData {
    device_id: "device1".to_string(),
    timestamp: Some(Timestamp::date_time(2026, 11, 16, 8, 0, 0).unwrap()),
    sensor_data: HashMap::from([("temperature".to_string(), 23.00)]),
});

static MOCK_TRANSPORT: LazyLock<MockUpstream> = LazyLock::new(|| MockUpstream {
    login_called: AtomicUsize::new(0),
    mock_data: &MOCK_DATA,
});

static INPUT: LazyLock<SensorDataRequest> = LazyLock::new(|| SensorDataRequest {
    device_id: "device1".to_string(),
    start: Some(Timestamp::date_time(2026, 11, 15, 8, 0, 0).unwrap()),
    stop: Some(Timestamp::date_time(2026, 11, 16, 8, 0, 0).unwrap()),
});

#[tokio::test(flavor = "current_thread")]
async fn test_lazy_login() {
    let cases = vec![
        LoginCase {
            name: "login on first request received",
            want_token: Token::new(String::from(common::VALID_TOKEN)),
            threads: 1,
        },
        LoginCase {
            name: "login once across multiple threads requiring login",
            want_token: Token::new(String::from(common::VALID_TOKEN)),
            threads: 5,
        },
        LoginCase {
            name: "login once across more threads requiring login",
            want_token: Token::new(String::from(common::VALID_TOKEN)),
            threads: 50,
        },
    ];
    for tc in cases {
        println!("case: {}", tc.name);
        let authed = upstream_api::Authed::new(&*MOCK_TRANSPORT);
        let futs = (0..tc.threads).map(|_| authed.sensor_data(tonic::Request::new(INPUT.clone())));
        let _results = futures::future::join_all(futs).await;
        assert_eq!(authed.token.read().unwrap().clone(), Some(tc.want_token));
        assert_eq!(MOCK_TRANSPORT.login_count(), 1);
        MOCK_TRANSPORT.reset_login_count();
    }
}

struct SensorDataCase {
    name: &'static str,
    want_code: tonic::Code,
    input: SensorDataRequest,
    want_empty: bool,
}

#[tokio::test(flavor = "current_thread")]
async fn test_sensor_data() {
    let cases = vec![
        SensorDataCase {
            name: "successfully get sensor data",
            want_empty: false,
            want_code: tonic::Code::Ok,
            input: SensorDataRequest {
                device_id: "device1".to_string(),
                start: Some(Timestamp::from(SystemTime::now())),
                stop: Some(Timestamp::date_time(2026, 11, 15, 8, 0, 0).unwrap()),
            },
        },
        SensorDataCase {
            name: "deviceId not found",
            want_empty: true,
            want_code: tonic::Code::NotFound,
            input: SensorDataRequest {
                device_id: "device2".to_string(),
                start: Some(Timestamp::from(SystemTime::now())),
                stop: Some(Timestamp::date_time(2026, 11, 15, 8, 0, 0).unwrap()),
            },
        },
        SensorDataCase {
            name: "data outside of timerange",
            want_empty: true,
            want_code: tonic::Code::Ok,
            input: SensorDataRequest {
                device_id: "device1".to_string(),
                start: Some(Timestamp::date_time(2025, 11, 15, 8, 0, 0).unwrap()),
                stop: Some(Timestamp::date_time(2025, 11, 16, 8, 0, 0).unwrap()),
            },
        },
    ];
    for tc in cases {
        println!("case: {}", tc.name);
        let authed = upstream_api::Authed::new(&*MOCK_TRANSPORT);
        let res = authed.sensor_data(tonic::Request::new(tc.input)).await;
        let (msg, code) = match res {
            Ok(resp) => (Some(resp.into_inner()), tonic::Code::Ok),
            Err(status) => (None, status.code()),
        };
        assert_eq!(tc.want_code, code);
        if !tc.want_empty {
            let sd_vec = msg.unwrap().response;
            assert_eq!(sd_vec, vec![MOCK_DATA.clone()]);
        }
    }
}
