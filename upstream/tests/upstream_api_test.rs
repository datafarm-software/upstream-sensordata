use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::{Arc, LazyLock};
use std::time::SystemTime;
use upstream::Authed;
use upstream::sensor_data::sensor_data_service_server::SensorDataService;
mod common;
use common::MockUpstream;
use entity::device_info;
use entity::prelude::DeviceInfo;
use prost_types::Timestamp;
use sea_orm::Database;
use std::sync::atomic::{AtomicUsize, Ordering};
use upstream::api::token::Token;
use upstream::device_info::Db;
use upstream::device_info::Mutation;
use upstream::sensor_data::{SensorData, SensorDataRequest};

struct LoginCase {
    name: &'static str,
    want_token: Token,
    starting_token: Token,
    threads: usize,
}

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
            starting_token: Token::new(String::from("")),
            threads: 1,
        },
        LoginCase {
            name: "login once across multiple threads requiring login",
            want_token: Token::new(String::from(common::VALID_TOKEN)),
            starting_token: Token::new(String::from("")),
            threads: 5,
        },
        LoginCase {
            name: "login once across more threads requiring login",
            want_token: Token::new(String::from(common::VALID_TOKEN)),
            starting_token: Token::new(String::from("")),
            threads: 50,
        },
        LoginCase {
            name: "login because current token is invalid",
            want_token: Token::new(String::from(common::VALID_TOKEN)),
            starting_token: Token::new(String::from(common::INVALID_TOKEN)),
            threads: 10,
        },
    ];
    let db = Database::connect("sqlite::memory:").await.unwrap();
    db.get_schema_builder()
        .register(DeviceInfo)
        .apply(&db)
        .await
        .unwrap();
    Mutation::create(
        &db,
        device_info::Model {
            id: 0,
            device_id: "device1".to_owned(),
            block_id: "block1".to_owned(),
        },
    )
    .await
    .unwrap();
    let shared_db = Arc::new(db);
    let login_called = Arc::new(AtomicUsize::new(0));
    let mock_data = Arc::new(SensorData {
        device_id: "device1".to_string(),
        timestamp: Some(Timestamp::date_time(2026, 11, 16, 8, 0, 0).unwrap()),
        sensor_data: HashMap::from([("temperature".to_string(), 23.00)]),
    });
    for tc in cases {
        println!("case: {}", tc.name);
        let mut authed = Authed::new(
            MockUpstream {
                login_called: login_called.clone(),
                mock_data: mock_data.clone(),
            },
            Arc::unwrap_or_clone(shared_db.clone()),
        )
        .await;
        authed.token = RwLock::new(Some(tc.starting_token));
        let futs = (0..tc.threads).map(|_| authed.sensor_data(tonic::Request::new(INPUT.clone())));
        let _results = futures::future::join_all(futs).await;
        assert_eq!(authed.token.read().unwrap().clone(), Some(tc.want_token));
        assert_eq!(login_called.clone().load(Ordering::SeqCst), 1);
        login_called.clone().store(0, Ordering::SeqCst);
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
                start: Some(Timestamp::date_time(2026, 11, 17, 8, 0, 0).unwrap()),
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
    ];

    let db = Database::connect("sqlite::memory:").await.unwrap();
    db.get_schema_builder()
        .register(DeviceInfo)
        .apply(&db)
        .await
        .unwrap();
    Mutation::create(
        &db,
        device_info::Model {
            id: 0,
            device_id: "device1".to_owned(),
            block_id: "block1".to_owned(),
        },
    )
    .await
    .unwrap();
    let shared_db = Arc::new(db);
    let login_called = Arc::new(AtomicUsize::new(0));
    let mock_data = Arc::new(SensorData {
        device_id: "device1".to_string(),
        timestamp: Some(Timestamp::date_time(2026, 11, 16, 8, 0, 0).unwrap()),
        sensor_data: HashMap::from([("temperature".to_string(), 23.00)]),
    });
    for tc in cases {
        println!("case: {}", tc.name);
        let authed = Authed::new(
            MockUpstream {
                login_called: login_called.clone(),
                mock_data: mock_data.clone(),
            },
            Arc::unwrap_or_clone(shared_db.clone()),
        )
        .await;
        let res = authed.sensor_data(tonic::Request::new(tc.input)).await;
        let (msg, code) = match res {
            Ok(resp) => (Some(resp.into_inner()), tonic::Code::Ok),
            Err(status) => (None, status.code()),
        };
        assert_eq!(tc.want_code, code);
        if !tc.want_empty {
            let sd_vec = msg.unwrap().response;
            assert_eq!(sd_vec, vec![Arc::unwrap_or_clone(mock_data.clone())]);
        }
    }
}

#[tokio::test]
async fn db_auto_reconnects() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    db.conn().await.close().await.unwrap();
    assert!(db.conn().await.ping().await.is_err());
    db.ensure_healthy().await.unwrap();
    assert!(db.conn().await.ping().await.is_ok());
}
