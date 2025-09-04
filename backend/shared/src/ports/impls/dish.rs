// In ports/dish.rs (or a submodule like ports/dish/http.rs)

use crate::api::{ApiEndpoint, DishInternalServiceApi, InternalApiError, SuperClient}; // DishInternalServiceApi is assumed to exist, similar to TrainInternalServiceApi
use crate::internal::dish::command::{SaveRawDishCommand, SaveRawTakeawayCommand};
use crate::internal::dish::dto::{DbDishDTO, DbTakeawayDishDTO};
use crate::ports::dish::DishPort;
use async_trait::async_trait;
use tracing::error;

/// An HTTP-based implementation of the `DishPort`.
///
/// This struct uses a `SuperClient` to make HTTP POST requests to the Dish microservice endpoints.
pub struct HttpDishPortImpl {
    super_client: SuperClient,
}

impl HttpDishPortImpl {
    /// Creates a new instance of `HttpDishPortImpl`.
    ///
    /// # Arguments
    ///
    /// * `api_endpoint` - The base endpoint configuration for the Dish microservice.
    pub fn new(api_endpoint: ApiEndpoint) -> Self {
        let super_client = SuperClient::new(api_endpoint);
        Self { super_client }
    }
}

#[async_trait]
impl DishPort for HttpDishPortImpl {
    async fn save_raw_dish(&self, command: SaveRawDishCommand) -> Result<(), InternalApiError> {
        self.super_client
            .post(DishInternalServiceApi::SaveRawDish, command)
            .await
            .inspect_err(|e| error!("Failed to save raw dish: {:?}", e))
    }

    async fn save_raw_takeaway(
        &self,
        command: SaveRawTakeawayCommand,
    ) -> Result<(), InternalApiError> {
        self.super_client
            .post(DishInternalServiceApi::SaveRawTakeaway, command)
            .await
            .inspect_err(|e| error!("Failed to save raw takeaway: {:?}", e))
    }

    async fn db_get_dishes(&self) -> Result<Vec<DbDishDTO>, InternalApiError> {
        self.super_client
            .get(DishInternalServiceApi::DbGetDishes)
            .await
            .inspect_err(|e| error!("Failed to get db dishes: {:?}", e))
    }

    async fn db_get_takeaway_dishes(&self) -> Result<Vec<DbTakeawayDishDTO>, InternalApiError> {
        self.super_client
            .get(DishInternalServiceApi::DbGetTakeawayDishes)
            .await
            .inspect_err(|e| error!("Failed to get db takeaway dishes: {:?}", e))
    }
}
