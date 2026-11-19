use ::entity::{device_info, device_info::Entity as DeviceInfo};
use sea_orm::*;

pub struct Query;
impl Query {
    pub async fn find_device_info(
        db: &DbConn,
        device_id: &String,
    ) -> Result<Option<device_info::Model>, DbErr> {
        DeviceInfo::find()
            .filter(device_info::COLUMN.device_id.contains(device_id))
            .one(db)
            .await
    }
}
