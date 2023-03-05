use async_trait::async_trait;
use crate::client::{StreamData, Streamer};

#[async_trait]
pub trait EventHandler: Send + Sync + 'static {
    async fn on_stream(&self, streamer: &Streamer, stream: &StreamData) {}
    async fn on_error(&self, error: String) {}
}