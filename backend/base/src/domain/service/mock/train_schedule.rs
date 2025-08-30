#![cfg(test)]

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::Verified;
use crate::domain::model::station::StationId;
use crate::domain::model::train::{TrainId, TrainNumber};
use crate::domain::model::train_schedule::{TrainSchedule, TrainScheduleId};
use crate::domain::service::train_schedule::{TrainScheduleService, TrainScheduleServiceError};
use sea_orm::prelude::DateTimeWithTimeZone;

use mockall::mock;

mock! {
    pub TrainScheduleService {}

    #[async_trait]
    impl TrainScheduleService for TrainScheduleService {
        async fn add_schedule(&self, train_id: TrainId, date: NaiveDate) -> Result<(), TrainScheduleServiceError>;

        async fn get_schedules(&self, date: NaiveDate) -> Result<Vec<TrainSchedule>, TrainScheduleServiceError>;

        async fn get_schedule_by_train_number_and_date(
            &self,
            train_number: String,
            departure_date: NaiveDate,
        ) -> Result<Option<TrainSchedule>, TrainScheduleServiceError>;

        async fn auto_plan_schedule(&self, begin_date: NaiveDate, days: i32) -> Result<(), TrainScheduleServiceError>;

        async fn auto_plan_schedule_daemon(&self, days: i32);

        async fn direct_schedules(
            &self,
            date: NaiveDate,
            pairs: &[(StationId, StationId)],
        ) -> Result<Vec<(TrainSchedule, StationId, StationId)>, TrainScheduleServiceError>;

        #[allow(clippy::type_complexity)]
        async fn transfer_schedules(
            &self,
            date: NaiveDate,
            pairs: &[(StationId, StationId)],
        ) -> Result<Vec<(Vec<TrainScheduleId>, StationId, StationId, Option<StationId>)>, TrainScheduleServiceError>;

        async fn get_station_arrival_time(
            &self,
            train_schedule_id: TrainScheduleId,
            station_id: StationId,
        ) -> Result<DateTimeWithTimeZone, TrainScheduleServiceError>;

        async fn get_terminal_arrival_time(
            &self,
            train_number: TrainNumber<Verified>,
            origin_departure_time: DateTimeWithTimeZone,
        ) -> Result<DateTimeWithTimeZone, TrainScheduleServiceError>;
    }
}

/// helper: 方便在测试里快速创建 mock
pub fn mock_train_schedule_service() -> MockTrainScheduleService {
    MockTrainScheduleService::new()
}
