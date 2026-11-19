use crate::device_info::{Mutation, Query};
use ::entity::device_info;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::result::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

pub async fn connect_with_retry(
    url: &str,
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
) -> Result<DatabaseConnection, DbErr> {
    let mut attempt = 0;
    loop {
        attempt += 1;
        let result = async {
            let mut opts = ConnectOptions::new(url.to_owned());
            opts.connect_timeout(Duration::from_secs(5));
            let db = Database::connect(opts).await?;
            db.ping().await?;
            Ok::<_, DbErr>(db)
        }
        .await;

        match result {
            Ok(db) => return Ok(db),
            Err(e) if attempt >= max_attempts => return Err(e),
            Err(_) => {
                let delay = base_delay
                    .saturating_mul(2u32.saturating_pow(attempt - 1))
                    .min(max_delay);
                sleep(delay).await;
            }
        }
    }
}

#[derive(Clone)]
pub struct Db {
    url: Arc<str>,
    conn: Arc<RwLock<DatabaseConnection>>,
}

impl Db {
    pub async fn new(url: &str) -> Result<Self, DbErr> {
        let conn = connect_with_retry(url, 10, Duration::from_millis(200), Duration::from_secs(10))
            .await?;
        Ok(Self {
            url: url.into(),
            conn: Arc::new(RwLock::new(conn)),
        })
    }

    pub async fn conn(&self) -> DatabaseConnection {
        self.conn.read().await.clone()
    }

    pub async fn conn_healthy(&self) -> Result<DatabaseConnection, DbErr> {
        if self.conn().await.ping().await.is_ok() {
            return Ok(self.conn.read().await.clone());
        }
        let fresh = connect_with_retry(
            &self.url,
            10,
            Duration::from_millis(200),
            Duration::from_secs(10),
        )
        .await?;
        *self.conn.write().await = fresh;
        Ok(self.conn.read().await.clone())
    }

    pub async fn find_device_info(
        &self,
        device_id: &String,
    ) -> Result<Option<device_info::Model>, DbErr> {
        let conn = self.conn_healthy().await?;
        Query::find_device_info(&conn, device_id).await
    }

    pub async fn create(&self, di: device_info::Model) -> Result<device_info::ActiveModel, DbErr> {
        let conn = self.conn_healthy().await?;
        Mutation::create(&conn, di).await
    }
}
