use async_trait::async_trait;
use shared::MicroService;
use shared::event::EventRegistry;
use shared::event::queue::{EventService, EventServiceError, get_channel};
use std::any::Any;
use std::sync::{Arc, Mutex};
use tracing::instrument;
use tracing::log::warn;

pub struct UserEventServiceImpl {
    channel: lapin::Channel,
    event_registry: Arc<Mutex<EventRegistry>>,
}

impl UserEventServiceImpl {
    pub async fn new(addr: &str) -> Result<Arc<Self>, EventServiceError> {
        let channel = get_channel(addr).await?;

        let event_registry = EventRegistry::new();

        Ok(Arc::new(Self {
            channel,
            event_registry: Arc::new(Mutex::new(event_registry)),
        }))
    }
}

#[async_trait]
impl EventService for UserEventServiceImpl {
    fn micro_service(&self) -> MicroService {
        MicroService::User
    }

    fn lapin_channel(&self) -> lapin::Channel {
        self.channel.clone()
    }

    fn event_registry(&self) -> Arc<Mutex<EventRegistry>> {
        Arc::clone(&self.event_registry)
    }

    #[instrument(skip_all)]
    async fn handle_event(
        &self,
        _event: Box<dyn Any + Send + Sync>,
    ) -> Result<(), EventServiceError> {
        warn!("Unknown event type");

        Ok(())
    }
}
