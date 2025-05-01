use crate::application::commands::train_data::{
    LoadCityCommand, LoadStationCommand, LoadTrainNumberCommand, LoadTrainTypeCommand,
};
use crate::application::service::train_data::TrainDataService;
use crate::application::{ApplicationError, GeneralError, ModeError};
use crate::domain::repository::city::CityRepository;
use crate::domain::repository::route::RouteRepository;
use crate::domain::repository::station::StationRepository;
use crate::domain::repository::train::TrainRepository;
use async_trait::async_trait;
use std::sync::Arc;

pub struct TrainDataServiceImpl<C, S, T, R>
where
    C: CityRepository,
    S: StationRepository,
    T: TrainRepository,
    R: RouteRepository,
{
    debug: bool,
    city_repository: Arc<C>,
    station_repository: Arc<S>,
    train_repository: Arc<T>,
    route_repository: Arc<R>,
}

impl<C, S, T, R> TrainDataServiceImpl<C, S, T, R>
where
    C: CityRepository,
    S: StationRepository,
    T: TrainRepository,
    R: RouteRepository,
{
    pub fn new(
        debug: bool,
        city_repository: Arc<C>,
        station_repository: Arc<S>,
        train_repository: Arc<T>,
        route_repository: Arc<R>,
    ) -> Self {
        Self {
            debug,
            city_repository,
            station_repository,
            train_repository,
            route_repository,
        }
    }

    pub fn check_debug_mode(&self) -> Result<(), Box<dyn ApplicationError>> {
        if self.debug {
            Ok(())
        } else {
            Err(Box::new(ModeError))
        }
    }
}

#[async_trait]
impl<C, S, T, R> TrainDataService for TrainDataServiceImpl<C, S, T, R>
where
    C: CityRepository,
    S: StationRepository,
    T: TrainRepository,
    R: RouteRepository,
{
    fn is_debug_mode(&self) -> bool {
        self.debug
    }

    async fn load_city(&self, command: LoadCityCommand) -> Result<(), Box<dyn ApplicationError>> {
        self.check_debug_mode()?;
        self.city_repository
            .save_raw(command)
            .await
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        Ok(())
    }

    async fn load_station(
        &self,
        command: LoadStationCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        self.check_debug_mode()?;

        self.station_repository
            .save_raw(command)
            .await
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        Ok(())
    }

    async fn load_train_type(
        &self,
        command: LoadTrainTypeCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        self.check_debug_mode()?;

        self.train_repository
            .save_raw_train_type(command)
            .await
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        Ok(())
    }

    async fn load_train_number(
        &self,
        command: LoadTrainNumberCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        self.check_debug_mode()?;

        self.train_repository
            .save_raw_train_number(command, Arc::clone(&self.route_repository))
            .await
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        Ok(())
    }
}
