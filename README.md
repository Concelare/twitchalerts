# Rust TwitchAlerts

A rust crate to allow the users to detect when a streamer is live a trigger a custom event. Rate Limiting is currently hardcoded at 80ms between individual checks but the delay between Check Cycles is fully customisable. Each streamer can currently be checked once every 30 seconds.

If you find any bugs or have a feature request, please report them on GitHub and any improvements and additions are welcome through pull requests 

## Features

- Stream Alerts
- Custom Delay
- Custom Error Handling

## Setup

The first run will create a config file, this should contain the client-id, token, delay and list of streamers to monitor.

To get your OAuth token for twitch go to https://dev.twitch.tv/console create an application and use the Client ID & Secret with the command below
```http request
curl -X POST 'https://id.twitch.tv/oauth2/token' \
-H 'Content-Type: application/x-www-form-urlencoded' \
-d 'client_id=<your client id goes here>&client_secret=<your client secret goes here>&grant_type=client_credentials'
```
### Example Config
```toml
streamers = ["streamer", "streamer2"]
delay = 80
token = "my_token"
user_id = "my_user_id"
```

## Example

```rust
use async_trait::async_trait;
use twitchalerts::client::{StreamData, Streamer, Client};
use twitchalerts::traits::EventHandler;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn on_stream(&self, streamer: &String, stream: &StreamData) {
        !todo();
    }

    async fn on_error(&self, error: String) {
        !todo();
    }
}

async fn main() -> Result<(), ()> {
    _ = Client::new("client id", "client token").event_handler(Handler).run().await?;

    Ok(())
}
```


## Dependencies

- [tokio](https://crates.io/crates/tokio)
- [serde](https://crates.io/crates/serde)
- [reqwest](https://crates.io/crates/reqwest)
- [chrono](https://crates.io/crates/chrono)
- [async-trait](https://crates.io/crates/async-trait)
- [toml](https://crates.io/crates/toml)

## Contributors
- [@ConcelareDev](https://www.github.com/ConcelareDev)

