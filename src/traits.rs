use async_trait::async_trait;
use crate::client::StreamData;
use crate::error::Error;

/// Trait used to Specify EventHandler
///
/// # Events
/// * `on_stream` - The Event Triggered When a Streamer Goes Live
/// * `on_error` - The Event Triggered When an Error Occurs
///
/// # Example
/// ```
/// use async_trait::async_trait;
/// use twitchalerts::client::{StreamData, Streamer};
/// use twitchalerts::traits::EventHandler;
///
/// pub struct Handler;
///
/// #[async_trait]
/// impl EventHandler for Handler {
///     async fn on_stream(&self, streamer: &Streamer, stream: &StreamData) {
///         println!("{} Has Gone Live", streamer.name);
///     }
///
///     async fn on_error(&self, error: String) {
///         println!("Error Occurred");
///     }
/// }
/// ```

#[async_trait]
pub trait EventHandler: Send + Sync + 'static {
    async fn on_stream(&self, _streamer: &String, _stream: &StreamData) {}
    async fn on_error(&self, _error: Error) {}
}