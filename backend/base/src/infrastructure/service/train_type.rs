use crate::domain::Identifiable;
use crate::domain::model::train::{SeatType, SeatTypeName, Train, TrainId, TrainNumber, TrainType};
use crate::domain::model::train_schedule::SeatId;
use crate::domain::repository::train::TrainRepository;
use crate::domain::service::train_type::{
    TrainTypeConfigurationService, TrainTypeConfigurationServiceError,
};
use crate::{Unverified, Verified};
use async_trait::async_trait;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

pub struct TrainTypeConfigurationServiceImpl<R>
where
    R: TrainRepository,
{
    train_repository: Arc<R>,
}

impl<R> TrainTypeConfigurationServiceImpl<R>
where
    R: TrainRepository,
{
    pub fn new(train_repository: Arc<R>) -> Self {
        Self { train_repository }
    }
}

#[async_trait]
impl<R> TrainTypeConfigurationService for TrainTypeConfigurationServiceImpl<R>
where
    R: TrainRepository,
{
    async fn verify_seat_type_name(
        &self,
        train_id: TrainId,
        seat_type_name: SeatTypeName<Unverified>,
    ) -> Result<SeatTypeName<Verified>, TrainTypeConfigurationServiceError> {
        let verified_seat_type = self
            .train_repository
            .get_verified_seat_type(train_id)
            .await?;

        if verified_seat_type.contains(seat_type_name.deref()) {
            Ok(SeatTypeName::from_unchecked(seat_type_name.to_string()))
        } else {
            Err(TrainTypeConfigurationServiceError::InvalidSeatType(
                train_id.into(),
                seat_type_name.to_string(),
            ))
        }
    }

    async fn verify_train_number(
        &self,
        train_number: TrainNumber<Unverified>,
    ) -> Result<TrainNumber<Verified>, TrainTypeConfigurationServiceError> {
        let verified_train_number = self.train_repository.get_verified_train_number().await?;

        if verified_train_number.contains(train_number.deref()) {
            Ok(TrainNumber::from_unchecked(train_number.to_string()))
        } else {
            Err(TrainTypeConfigurationServiceError::InvalidTrainNumber(
                train_number.to_string(),
            ))
        }
    }

    async fn verify_train_type(
        &self,
        train_type: TrainType<Unverified>,
    ) -> Result<TrainType<Verified>, TrainTypeConfigurationServiceError> {
        let verified_train_type = self.train_repository.get_verified_train_type().await?;

        if verified_train_type.contains(train_type.deref()) {
            Ok(TrainType::from_unchecked(train_type.to_string()))
        } else {
            Err(TrainTypeConfigurationServiceError::InvalidTrainType(
                train_type.to_string(),
            ))
        }
    }

    async fn get_seat_id_map(
        &self,
        train_id: TrainId,
    ) -> Result<HashMap<SeatTypeName<Verified>, Vec<SeatId>>, TrainTypeConfigurationServiceError>
    {
        if self.train_repository.find(train_id).await?.is_some() {
            let result = self.train_repository.get_seat_id_map(train_id).await?;

            Ok(result)
        } else {
            Err(TrainTypeConfigurationServiceError::NoSuchTrainId(
                train_id.into(),
            ))
        }
    }

    async fn get_trains(&self) -> Result<Vec<Train>, TrainTypeConfigurationServiceError> {
        let result = self.train_repository.get_trains().await?;

        Ok(result)
    }

    async fn get_train_by_number(
        &self,
        train_number: TrainNumber<Verified>,
    ) -> Result<Train, TrainTypeConfigurationServiceError> {
        let result = self
            .train_repository
            .find_by_train_number(train_number)
            .await?;

        Ok(result)
    }

    async fn add_train_type(
        &self,
        train_number: TrainNumber<Verified>,
        train_type: TrainType<Verified>,
        seat_configuration: Vec<SeatType>,
    ) -> Result<TrainId, TrainTypeConfigurationServiceError> {
        let seat_map = seat_configuration
            .into_iter()
            .map(|x| (x.name().to_string(), x))
            .collect::<HashMap<_, _>>();

        let mut train = Train::new(None, train_number, train_type, seat_map);

        self.train_repository.save(&mut train).await?;

        Ok(train.get_id().expect("saved train should have id"))
    }

    async fn modify_train_type(
        &self,
        train_id: TrainId,
        train_number: TrainNumber<Verified>,
        train_type: TrainType<Verified>,
        seat_configuration: Vec<SeatType>,
    ) -> Result<(), TrainTypeConfigurationServiceError> {
        if self.train_repository.find(train_id).await?.is_some() {
            let seat_map = seat_configuration
                .into_iter()
                .map(|x| (x.name().to_string(), x))
                .collect::<HashMap<_, _>>();

            let mut train = Train::new(None, train_number, train_type, seat_map);

            self.train_repository.save(&mut train).await?;

            Ok(())
        } else {
            Err(TrainTypeConfigurationServiceError::NoSuchTrainId(
                train_id.into(),
            ))
        }
    }

    async fn remove_train_type(
        &self,
        train: Train,
    ) -> Result<(), TrainTypeConfigurationServiceError> {
        self.train_repository.remove(train).await?;

        Ok(())
    }
}
