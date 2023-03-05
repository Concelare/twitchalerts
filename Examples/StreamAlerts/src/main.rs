use async_trait::async_trait;
use chrono::Utc;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;
use twitchalerts::client::{StreamData, Streamer, Client};

pub struct Handler;

#[async_trait]
impl crate::traits::EventHandler for Handler {
    async fn on_stream(&self, streamer: &Streamer, stream: &StreamData) {
        println!("{} Has Gone Live", streamer.name);
    }

    async fn on_error(&self, error: String) {
        println!("Error Occurred");
    }
}

async fn main() -> Result<(), ()> {
    let db = Surreal::new::<Mem>(()).await?;

    db.use_ns("namespace").use_db("database").await?;

    let streamer: Streamer = Streamer {
        id: "".to_string(),
        name: "example_streamer".to_string(),
        alerts: true,
        last_streamed: Utc::now(),
    };

    db.query("CREATE streamers SET name = $name, alerts = $alerts, last_streamed = $last_streamed").bind(&streamer).await?;

    _ = Client::new("client id", "client token").database(db).event_handler(Handler).run().await?;

    Ok(())
}