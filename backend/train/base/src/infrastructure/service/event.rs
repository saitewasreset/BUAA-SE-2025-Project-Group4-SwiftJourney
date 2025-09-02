use async_trait::async_trait;
use shared::MicroService;
use shared::event::EventRegistry;
use shared::event::queue::{EventService, EventServiceError, get_channel};
use std::any::Any;
use std::sync::{Arc, Mutex};

pub struct TrainEventServiceImpl {
    channel: lapin::Channel,
}

impl TrainEventServiceImpl {
    pub fn new(addr: &str) -> Result<Arc<Self>, EventServiceError> {
        let channel = get_channel(addr)?;

        Ok(Arc::new(Self { channel }))
    }
}

#[async_trait]
impl EventService for TrainEventServiceImpl {
    fn micro_service(&self) -> MicroService {
        MicroService::Train
    }

    fn lapin_channel(&self) -> lapin::Channel {
        self.channel.clone()
    }

    fn event_registry(&self) -> Arc<Mutex<EventRegistry>> {
        todo!()
    }

    async fn handle_event(
        &self,
        event: Box<dyn Any + Send + Sync>,
    ) -> Result<(), EventServiceError> {
        todo!()
    }
}
