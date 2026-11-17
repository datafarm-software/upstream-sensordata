use ::entity::device_info;

use sea_orm::*;

pub struct Mutation;

impl Mutation {
    pub async fn create(
        db: &DbConn,
        di: device_info::Model,
    ) -> Result<device_info::ActiveModel, std::error::Error> {
        device_info::ActiveModel {
            device_id: Set(di.device_id.to_owned()),
            block_id: Set(di.block_id.to_owned()),
            ..Default::default()
        }
        .save(db)
        .await
    }
}
