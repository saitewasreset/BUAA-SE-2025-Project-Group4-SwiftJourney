use crate::domain::model::station::StationId;
use crate::domain::model::train::TrainId;
use crate::domain::model::train_schedule::TrainSchedule;
use crate::domain::service::train_schedule::{TrainScheduleService, TrainScheduleServiceError};
use async_trait::async_trait;
use chrono::NaiveDate;

// Step 1: Define generics parameter over `RouteService` service
// Exercise 1.2.1D - 3: Your code here. (1 / 6)
pub struct TrainScheduleServiceImpl {
    // Step 2: Add struct filed to store an implementation of `RouteService` service
    // Exercise 1.2.1D - 3: Your code here. (2 / 6)
}

#[async_trait]
impl TrainScheduleService for TrainScheduleServiceImpl {
    async fn add_schedule(
        &self,
        train_id: TrainId,
        date: NaiveDate,
    ) -> Result<(), TrainScheduleServiceError> {
        todo!()
    }

    async fn get_schedules(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<TrainSchedule>, TrainScheduleServiceError> {
        todo!()
    }

    async fn find_schedules(
        &self,
        date: NaiveDate,
        from_station: StationId,
        to_station: StationId,
    ) -> Result<Vec<TrainSchedule>, TrainScheduleServiceError> {
        // Step 3: Load route map using `get_route_map` from stored `RouteService` implementation
        // Exercise 1.2.1D - 3: Your code here. (3 / 6)

        // Step 4: Check if `from_station` and `to_station` are valid
        // if not, return Err(TrainScheduleServiceError::InvalidStationId)
        // HINT: you can check if `from_station` and `to_station` present in your RouteMap

        // Exercise 1.2.1D - 3: Your code here. (4 / 6)

        // Step 5: Find k-shortest path from `from_station` to `to_station`
        // HINT: you may refer to algorithm in https://docs.rs/petgraph/latest/petgraph/algo/index.html

        // Exercise 1.2.1D - 3: Your code here. (5 / 6)

        // Step 6: Filter output to contain only valid schedules for specified `date`
        // HINT: you can get valid schedules for specified `date` using `self.get_schedules`
        // Exercise 1.2.1D - 3: Your code here. (6 / 6)
        // Good! Next, define your `TrainQueryService` application service in `base::application::service::train_query`
        todo!()
    }
}
