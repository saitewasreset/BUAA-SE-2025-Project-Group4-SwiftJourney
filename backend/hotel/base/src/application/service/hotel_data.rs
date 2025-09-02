use crate::application::commands::hotel_data::LoadHotelCommand;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use shared::application_error::ApplicationError;

#[async_trait]
pub trait HotelDataService: 'static + Send + Sync {
    /// 检查服务是否处于调试模式
    fn is_debug_mode(&self) -> bool;

    /// 加载酒店数据
    ///
    /// 根据[`LoadHotelCommand`]提供的信息创建或更新酒店数据。
    ///
    /// # Arguments
    /// * `command` - 包含酒店数据的命令对象
    async fn load_hotel(
        &self,
        command: LoadHotelCommand,
        db: &DatabaseConnection,
    ) -> Result<(), Box<dyn ApplicationError>>;
}
