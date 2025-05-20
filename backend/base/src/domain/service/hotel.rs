use async_trait::async_trait;

#[async_trait]
pub trait HotelService: 'static + Send + Sync {}
