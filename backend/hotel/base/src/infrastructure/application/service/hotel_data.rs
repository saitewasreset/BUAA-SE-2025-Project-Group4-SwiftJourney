use crate::application::commands::hotel_data::LoadHotelCommand;
use crate::application::service::hotel_data::HotelDataService;
use crate::infrastructure::repository::hotel::save_raw_hotel;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use shared::MicroService;
use shared::application_error::{ApplicationError, GeneralError};
use shared::event::queue::EventService;
use shared::event::{EventPackage, HotelRoomTypeUpdatedEvent, HotelUpdatedEvent};
use shared::ports::geo::GeoPort;
use shared::ports::object_storage::ObjectStoragePort;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, instrument};

pub struct HotelDataServiceImpl<GP, OP, ES>
where
    GP: GeoPort,
    OP: ObjectStoragePort,
    ES: EventService,
{
    debug: bool,
    data_base_path: PathBuf,
    geo_port: Arc<GP>,
    object_storage_port: Arc<OP>,
    event_service: Arc<ES>,
}

impl<GP, OP, ES> HotelDataServiceImpl<GP, OP, ES>
where
    GP: GeoPort,
    OP: ObjectStoragePort,
    ES: EventService,
{
    pub fn new(
        debug: bool,
        data_base_path: PathBuf,
        geo_port: Arc<GP>,
        object_storage_port: Arc<OP>,
        event_service: Arc<ES>,
    ) -> Self {
        HotelDataServiceImpl {
            debug,
            data_base_path,
            geo_port,
            object_storage_port,
            event_service,
        }
    }
}

#[async_trait]
impl<GP, OP, ES> HotelDataService for HotelDataServiceImpl<GP, OP, ES>
where
    GP: GeoPort,
    OP: ObjectStoragePort,
    ES: EventService,
{
    fn is_debug_mode(&self) -> bool {
        self.debug
    }

    #[instrument(skip_all)]
    async fn load_hotel(
        &self,
        command: LoadHotelCommand,
        db: &DatabaseConnection,
    ) -> Result<(), Box<dyn ApplicationError>> {
        save_raw_hotel(
            Arc::clone(&self.object_storage_port),
            Arc::clone(&self.geo_port),
            db,
            &self.data_base_path,
            command,
        )
        .await
        .map_err(|e| {
            error!("Failed to save raw hotel data: {}", e);

            Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
        })?;

        if let Err(e) = self
            .event_service
            .publish_event(EventPackage::new(MicroService::Hotel, HotelUpdatedEvent))
            .await
        {
            error!("Failed to publish hotel updated event: {:?}", e);
        }

        if let Err(e) = self
            .event_service
            .publish_event(EventPackage::new(
                MicroService::Hotel,
                HotelRoomTypeUpdatedEvent,
            ))
            .await
        {
            error!("Failed to publish hotel room type updated event: {:?}", e);
        }

        Ok(())
    }
}
