#![cfg(test)]

use async_trait::async_trait;
use std::collections::HashMap;

use crate::domain::model::route::RouteId;
use crate::domain::model::train::{SeatType, SeatTypeName, Train, TrainId, TrainNumber, TrainType};
use crate::domain::model::train_schedule::{SeatId, SeatLocationInfo};
use crate::domain::service::train_type::{
    TrainTypeConfigurationService, TrainTypeConfigurationServiceError,
};
use crate::{Unverified, Verified};

use mockall::mock;

// 使用 mockall 创建 TrainTypeConfigurationService 的 Mock
mock! {
    pub TrainTypeConfigurationService {}

    #[async_trait]
    impl TrainTypeConfigurationService for TrainTypeConfigurationService {
        async fn verify_seat_type_name(
            &self,
            train_id: TrainId,
            seat_type_name: SeatTypeName<Unverified>
        ) -> Result<SeatTypeName<Verified>, TrainTypeConfigurationServiceError>;

        async fn verify_train_number(
            &self,
            train_number: TrainNumber<Unverified>
        ) -> Result<TrainNumber<Verified>, TrainTypeConfigurationServiceError>;

        async fn verify_train_type(
            &self,
            train_type: TrainType<Unverified>
        ) -> Result<TrainType<Verified>, TrainTypeConfigurationServiceError>;

        #[allow(clippy::type_complexity)]
        async fn get_seat_id_map(
            &self,
            train_id: TrainId
        ) -> Result<HashMap<SeatTypeName<Verified>, Vec<(SeatId, SeatLocationInfo)>>, TrainTypeConfigurationServiceError>;

        async fn get_trains(
            &self
        ) -> Result<Vec<Train>, TrainTypeConfigurationServiceError>;

        async fn get_train_by_number(
            &self,
            train_number: TrainNumber<Verified>
        ) -> Result<Train, TrainTypeConfigurationServiceError>;

        async fn add_train_type(
            &self,
            train_number: TrainNumber<Verified>,
            train_type: TrainType<Verified>,
            seat_configuration: Vec<SeatType>,
            default_route_id: RouteId,
            default_origin_departure_time: i32
        ) -> Result<TrainId, TrainTypeConfigurationServiceError>;

        async fn modify_train_type(
            &self,
            train_id: TrainId,
            train_number: TrainNumber<Verified>,
            train_type: TrainType<Verified>,
            seat_configuration: Vec<SeatType>,
            default_route_id: RouteId,
            default_origin_departure_time: i32
        ) -> Result<(), TrainTypeConfigurationServiceError>;

        async fn remove_train_type(
            &self,
            train: Train
        ) -> Result<(), TrainTypeConfigurationServiceError>;
    }
}

// Helper 函数，方便在测试中生成 Arc<MockTrainTypeConfigurationService>
pub fn mock_train_type_service() -> MockTrainTypeConfigurationService {
    MockTrainTypeConfigurationService::new()
}
