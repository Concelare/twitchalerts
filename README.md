# TwitchAlerts

A rust crate to allow the users to detect when a streamer is live a trigger a custom event. Requires a Surreal Database but does support Memory and File Surreal Databases.  Rate Limiting is currently hardcoded but will be made dynamic later on.


## Features

- Stream Alerts
- More Coming


## Example

```rust
use async_trait::async_trait;
use chrono::Utc;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;
use twitchalerts::client::{StreamData, Streamer, Client};
use twitchalerts::traits::EventHandler;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn on_stream(&self, streamer: &Streamer, stream: &StreamData) {
        !todo();
    }

    async fn on_error(&self, error: String) {
        !todo();
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
```


## Dependencies

- [surrealdb](https://crates.io/crates/surrealdb)
- [tokio](https://crates.io/crates/tokio)
- [serde](https://crates.io/crates/serde)
- [reqwest](https://crates.io/crates/reqwest)
- [chrono](https://crates.io/crates/chrono)
- [async-trait](https://crates.io/crates/async-trait)

## Authors

- [@DeathsCookie](https://www.github.com/DeathsCookie)

