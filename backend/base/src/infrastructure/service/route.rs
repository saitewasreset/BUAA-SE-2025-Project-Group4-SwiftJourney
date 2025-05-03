use crate::domain::model::route::{Route, RouteId, Stop};
use crate::domain::model::train::Train;
use crate::domain::service::route::{RouteGraph, RouteService, RouteServiceError};
use async_trait::async_trait;

// Step 1: Define generics parameter over `TrainTypeConfigurationService` service
// Exercise 1.2.1D - 2: Your code here. (1 / 4)
pub struct RouteServiceImpl {
    // Step 2: Add struct filed to store an implementation of `TrainTypeConfigurationService` service
    // Exercise 1.2.1D - 2: Your code here. (2 / 4)
}

#[async_trait]
impl RouteService for RouteServiceImpl {
    async fn get_route_map(&self) -> Result<RouteGraph, RouteServiceError> {
        // Step 3: Load route data using `get_routes`

        let routes = self.get_routes().await?;
        let trains: Vec<Train>;

        // Step 4: Load train data using `get_trains` from stored `TrainTypeConfigurationService` implementation
        // Exercise 1.2.1D - 2: Your code here. (3 / 4)

        // Step 5: Build the graph using `routes` and `trains`
        // HINT: you may refer to https://docs.rs/petgraph/latest/petgraph/
        // Exercise 1.2.1D - 2: Your code here. (4 / 4)
        // Good! Next, implement `find_schedules` function in `base::infrastructure::service::train_schedule`
        todo!()
    }

    async fn add_route(&self, stops: Vec<Stop>) -> Result<RouteId, RouteServiceError> {
        todo!()
    }

    async fn get_routes(&self) -> Result<Vec<Route>, RouteServiceError> {
        todo!()
    }
}
