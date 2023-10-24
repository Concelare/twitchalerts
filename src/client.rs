use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tokio::task;
use crate::config::Config;
use crate::traits::EventHandler;

/// Stores streamers who are currently streaming so that the event doesn't repeatedly trigger
static mut C_STREAMING: Vec<String> = Vec::new();

/// All Streamer info store in Config
///
/// # Parameters
/// * `id` - Database identifier
/// * `name` - The Twitch streamer's name for checking
/// * `alerts` - Are the alerts enabled for the streamer
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Streamer {
    pub name: String
}

/// The Response from the checking request
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct StreamsRes {
    pub data: Vec<StreamData>,
    pub pagination: Pagination
}

/// The data for the stream
///
/// # Parameters
/// * `id` - Stream Identifier
/// * `user_id` - Streamer's User ID
/// * `user_login` - Streamer's User Login Name
/// * `user_name` - Streamer's Username
/// * `game_id` - The Game Identifier
/// * `game_name` - The Name of the Game
/// * `stream_type` - Type of the stream
/// * `title` - Title of the stream
/// * `viewer_count` - The Viewer Count
/// * `started_at` - The Time the Stream Started
/// * `language` - Language of the Stream
/// * `thumbnail_url` - Thumbnail of the Stream
/// * `tags_ids` - IDs of the Tags Used
/// * `tags` - Tags of the Stream
/// * `is_mature` - Is the Stream Set As Mature
#[derive(Serialize, Deserialize, Debug)]
pub struct StreamData {
    pub id: String,
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
    pub game_id: String,
    pub game_name: String,
    #[serde(rename = "type")]
    pub stream_type: String,
    pub title: String,
    pub viewer_count: u32,
    pub started_at: DateTime<Utc>,
    pub language: String,
    pub thumbnail_url: String,
    pub tags_ids: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub is_mature: bool
}

/// Part of the Response From Twitch
///
/// # Parameters
/// * `cursor` - Cursor String
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Pagination {
    pub cursor: String
}


/// Client For Running TwitchAlerts
///
/// # Parameters
/// * `client_id` - Twitch Client ID, can be got from <https://dev.twitch.tv/>
/// * `token` - Twitch Token, can be got from <https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/
/// * `event_handler` - The Event Handler To Handle Alerts and Errors
/// * `config` - The Config for the Client
/// * `currently_streaming` - The streamers that are currently streaming
/// * `delay` - Delay Between Check Cycles
#[derive(Clone)]
pub struct Client {
    pub client_id: String,
    pub token: String,
    event_handler: Option<Arc<dyn EventHandler>>,
    config: Config,
    currently_streaming: Vec<String>,
    delay: tokio::time::Duration
}

impl Client {
    /// Used to Create A New Instance of TwitchAlerts Client
    ///
    /// # Arguments
    ///
    /// * `client_id` - A &str of your Twitch Client ID which can be found at <https://dev.twitch.tv/>
    /// * `token` - A &str of your Twitch Token which can be found at <https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/>
    ///
    /// # Example
    /// ```
    /// use twitchalerts::client::Client;
    ///
    /// let client: Client = Client::new();
    /// ```
    pub async fn new() -> Client {

        let c = crate::config::read_config().await;

        if c.user_id.clone().is_none() {
            panic!("Missing User ID in Config File!")
        }
        if c.token.clone().is_none() {
            panic!("Missing User Token in Config File!")
        }

        let mut d = c.delay.clone().unwrap() as u64;

        if d < 80u64 {
            d = 80u64;
        }

        Client {
            client_id: c.user_id.clone().unwrap(),
            token: c.token.clone().unwrap(),
            event_handler: None,
            config: c.clone(),
            currently_streaming: Vec::new(),
            delay: tokio::time::Duration::from_millis(d)
        }
    }

    /// Used to add the Event Handler to the Client
    ///
    /// # Arguments
    /// * `self` - Requires a Client To Run The Function
    /// * `event_handler` - Requires a Struct With the EventHandler Trait and it must be Static
    ///
    /// # Example
    /// ```
    /// use async_trait::async_trait;
    /// use twitchalerts::client::{Client, StreamData, Streamer};
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
    ///
    /// let client: Client = Client::new().event_handler(Handler);
    /// ```
    pub fn event_handler<H: EventHandler + 'static>(mut self, event_handler: H) -> Self {
        self.event_handler = Some(Arc::new(event_handler));

        self
    }

    /// Used to start running the TwitchAlerts Client
    ///
    ///  # Arguments
    /// * `self` - Requires a Client To Run The Function
    ///
    /// # Example
    /// ```
    /// use async_trait::async_trait;
    /// use chrono::Utc;
    /// use twitchalerts::client::{StreamData, Streamer, Client};
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
    ///
    /// async fn main() -> Result<(), ()> {
    ///      _ = Client::new().event_handler(Handler).run().await?;
    ///
    ///     Ok(())
    /// }
    ///```
    pub async fn run(self) -> Result<(), crate::error::Error> {
        if self.event_handler.is_none() {
            panic!("No Event Handler Set");
        }

        let mut recent: HashMap<String, DateTime<Utc>> = HashMap::new();
        let mut running = true;


        while running {
            let mut local_client: Client = self.clone();

            tokio::time::sleep(self.delay.clone()).await;


            let streamers: Vec<String> = local_client.config.streamers.clone();

            if streamers.is_empty() {
                running = false;
            }


            for streamer in streamers {

                if let Some(time) = recent.get(streamer.as_str()) {
                    let difference: Duration = Utc::now() - *time;
                    if 30 > difference.num_seconds() {
                        continue;
                    }
                    else {
                        recent.remove(streamer.as_str());
                    }
                }

                recent.insert(streamer.clone(), Utc::now());

                let handler = local_client.event_handler.clone().unwrap();
                let t_string = local_client.token.clone();
                let u_string = local_client.client_id.clone();

                tokio::spawn(async move {
                    let client = reqwest::Client::new();


                    let res = client.get(format!("https://api.twitch.tv/helix/streams?user_login={0}", streamer.clone()))
                        .bearer_auth(t_string.clone()).header("Client-Id", u_string.clone()).send().await.expect("Error Occurred");

                    let rjson = res.json::<StreamsRes>().await;

                    match rjson {
                        Ok(json) => unsafe {
                            if json.data.is_empty() {
                                return;
                            }

                            let info = json.data.first().expect("Missing Info");

                            if C_STREAMING.contains(&info.user_id) {
                                return;
                            }

                            C_STREAMING.push(info.user_id.clone());
                            handler.on_stream(&streamer, info).await;
                        },
                        Err(e) => unsafe {
                            if e.is_timeout() {
                                handler.on_error(crate::error::Error::new("An error occurred due to timing out...", 1u16)).await;
                            } else if e.is_connect() {
                                handler.on_error(crate::error::Error::new("An error occurred when trying to connect...", 2u16)).await;
                            } else if e.is_status() {
                                handler.on_error(crate::error::Error::new("Status returned as an Error...", 3u16)).await;
                            } else if e.is_redirect() {
                                handler.on_error(crate::error::Error::new("An error occurred due to an attempted redirect...", 4u16)).await;
                            } else if e.is_request() {
                                handler.on_error(crate::error::Error::new("An error occurred due to the request...", 5u16)).await;
                            } else if e.is_body() {
                                handler.on_error(crate::error::Error::new("An error occurred with the request or response body...", 6u16)).await;
                            } else if e.is_builder() {
                                handler.on_error(crate::error::Error::new("An error occurred with the type builder...", 7u16)).await;
                            } else {
                                if C_STREAMING.contains(&streamer) {
                                    C_STREAMING.retain(|x | x.to_string() != streamer)
                                }
                            }
                        }
                    }
                    task::yield_now().await;
                }).await.expect("TODO: panic message");

                tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
            }
        };
        Ok(())
    }
}