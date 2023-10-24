use async_trait::async_trait;
use chrono::Utc;
use twitchalerts::client::{StreamData, Streamer, Client};
use twitchalerts::traits::EventHandler;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn on_stream(&self, streamer: &Streamer, stream: &StreamData) {
        println!("{} Has Gone Live", streamer.name);
    }

    async fn on_error(&self, error: String) {
        println!("Error Occurred");
    }
}

async fn main() -> Result<(), ()> {
       _ = Client::new().event_handler(Handler).run().await?;

    Ok(())
}