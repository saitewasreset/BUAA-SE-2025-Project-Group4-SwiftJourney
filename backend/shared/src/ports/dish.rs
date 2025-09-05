// In ports/dish.rs

use crate::api::InternalApiError;
use crate::internal::dish::command::{SaveRawDishCommand, SaveRawTakeawayCommand};
use crate::internal::dish::dto::{DbDishDTO, DbTakeawayDishDTO, DbTakeawayShopDTO};
use async_trait::async_trait;

/// Trait defining the port for communicating with the Dish internal service.
///
/// This acts as an abstraction layer, allowing different implementations (e.g., HTTP, gRPC, or mocks for testing)
/// for how other microservices interact with the Dish service.
#[async_trait]
pub trait DishPort: 'static + Send + Sync {
    /// Sends a command to save a raw dish entity.
    async fn save_raw_dish(&self, command: SaveRawDishCommand) -> Result<(), InternalApiError>;

    /// Sends a command to save a raw takeaway entity.
    async fn save_raw_takeaway(
        &self,
        command: SaveRawTakeawayCommand,
    ) -> Result<(), InternalApiError>;

    async fn db_get_dishes(&self) -> Result<Vec<DbDishDTO>, InternalApiError>;

    async fn db_get_takeaway_dishes(&self) -> Result<Vec<DbTakeawayDishDTO>, InternalApiError>;

    async fn db_get_takeaway_shops(&self) -> Result<Vec<DbTakeawayShopDTO>, InternalApiError>;
}
