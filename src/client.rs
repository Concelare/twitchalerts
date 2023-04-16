use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Duration, Utc};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use serde::{Deserialize, Serialize};
use crate::traits::EventHandler;

/// All Streamer info store in Database
///
/// # Parameters
/// * `id` - Database identifier
/// * `name` - The Twitch streamer's name for checking
/// * `alerts` - Are the alerts enabled for the streamer
/// * `last_streamed` - Stores the date of the last started stream to stop duplicate alerts
#[derive(Serialize, Deserialize, PartialEq)]
pub struct Streamer {
    pub id: String,
    pub name: String,
    pub alerts: bool,
    pub last_streamed: DateTime<Utc>,
}

/// The Response from the checking request
#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
pub(crate) struct Pagination {
    pub cursor: String
}


/// Client For Running TwitchAlerts
///
/// # Parameters
/// * `client_id` - Twitch Client ID, can be got from <https://dev.twitch.tv/>
/// * `token` - Twitch Token, can be got from <https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/>
/// * `status` - Status of alerts, are the running or not
/// * `event_handler` - The Event Handler To Handle Alerts and Erros
/// * `database` - The SurrealDb for the Client
/// * `delay` - Delay Between Check Cycles
#[derive(Clone)]
pub struct Client {
    pub client_id: String,
    pub token: String,
    event_handler: Option<Arc<dyn EventHandler>>,
    database: Option<Surreal<Db>>,
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
    /// let client: Client = Client::new("client id", "client token");
    /// ```
    pub fn new(client_id: &str, token: &str) -> Client {
        Client {
            client_id: client_id.to_string(),
            token: token.to_string(),
            event_handler: None,
            database: None::<Surreal<Db>>,
            delay: tokio::time::Duration::from_millis(500)
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
    /// let client: Client = Client::new("client id", "client token").event_handler(Handler);
    /// ```
    pub fn event_handler<H: EventHandler + 'static>(mut self, event_handler: H) -> Self {
        self.event_handler = Some(Arc::new(event_handler));

        self
    }

    /// Used to Add The SurrealDB to The Client
    ///
    /// # Arguments
    /// * `self` - Requires a Client To Run The Function
    /// * `database` - An Instance of Surreal<Db>
    ///
    /// # Examples
    /// ```
    /// use surrealdb::Surreal;
    /// use twitchalerts::client::Client;
    /// use surrealdb::engine::local::{Mem, Db};
    ///
    /// async fn main() -> Result<(), ()>  {
    ///     let db: Surreal<Db> = Surreal::new::<Mem>(()).await?;
    ///     let client: Client = Client::new("client id", "client token").database(db);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn database(mut self, database: Surreal<Db>) -> Self {
        self.database = Some(database);

        self
    }

    /// Sets the Delay Between Each Check Cycle
    /// Individual Checks are hard coded at 80ms
    /// The Individual Check delay is 80ms as Twitch RateLimit is set at 800 Requests a Minute
    ///
    /// # Arguments
    /// * `self` - Requires a Client To Run The Function
    /// * `delay` - Requires a Tokio Duration to Set Delay Between Check Cycles
    ///
    /// # Examples
    /// ```
    /// use twitchalerts::client::Client;
    ///
    /// let delay = tokio::time::Duration::from_secs(15);
    /// let client: Client = Client::new("client id", "client token").set_delay(delay);
    ///
    /// ```
    pub fn set_delay(mut self, delay: tokio::time::Duration) -> Self {
        self.delay = delay;

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
    /// use surrealdb::engine::local::Mem;
    /// use surrealdb::Surreal;
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
    ///     let db = Surreal::new::<Mem>(()).await?;
    ///
    ///     db.use_ns("namespace").use_db("database").await?;
    ///
    ///     let streamer: Streamer = Streamer {
    ///         id: "".to_string(),
    ///         name: "example_streamer".to_string(),
    ///         alerts: true,
    ///         last_streamed: Utc::now(),
    ///     };
    ///
    ///     db.query("CREATE streamers SET name = $name, alerts = $alerts, last_streamed = $last_streamed").bind(&streamer).await?;
    ///
    ///     _ = Client::new("client id", "client token").database(db).event_handler(Handler).run().await?;
    ///
    ///     Ok(())
    /// }
    ///```
    pub async fn run(self) -> Result<(), crate::error::Error> {
        if self.event_handler.is_none() {
            panic!("No Event Handler Set");
        }

        if self.database.is_none() {
            panic!("No Database Set");
        }

        let local_client: Client = self.clone();

        let mut recent: HashMap<String, DateTime<Utc>> = HashMap::new();
        let mut running = true;


        while running {
            tokio::time::sleep(self.delay.clone()).await;

            let mut res = local_client.database.as_ref().expect("Error Occurred").query("SELECT * FROM streamers WHERE alerts = true").await.expect("Error Occurred");

            let streamers: Vec<Streamer> = res.take(0).expect("Error Occurred");

            if streamers.is_empty() {
                running = false;
            }

            for streamer in streamers {
                if let Some(time) = recent.get(streamer.name.as_str()) {
                    let difference: Duration = Utc::now() - *time;
                    if 30 > difference.num_seconds() {
                        continue;
                    }
                    else {
                        recent.remove(streamer.name.as_str());
                    }
                }

                recent.insert(streamer.name.clone(), Utc::now());

                let client_cloned: Client = self.clone();
                let handler = match self.event_handler.clone() {
                    Some(evh) => evh,
                    _ => {panic!("No Event Handler Found");}
                };

                let result = tokio::spawn(async move {
                    let client = reqwest::Client::new();

                    let res = client.get(format!("https://api.twitch.tv/helix/streams?user_login={0}", streamer.name.clone()))
                        .bearer_auth(client_cloned.token.clone()).header("Client-Id", client_cloned.client_id.clone()).send().await.expect("Error Occurred");

                    let rjson = res.json::<StreamsRes>().await;

                    match rjson {
                        Ok(json) => {
                            if json.data.is_empty() {
                                return;
                            }

                            let info = json.data.first().expect("Missing Info");

                            if info.started_at == streamer.last_streamed {
                                return;
                            }

                            _ = client_cloned.database.expect("Missing Database").query("UPDATE streamers SET last_streamed = $last_streamed WHERE name = $name").bind(("last_streamed", info.started_at.clone())).bind(("name", streamer.name.clone())).await.expect("Error Occurred");
                            handler.on_stream(&streamer, info).await;
                        },
                        Err(e) => {
                            if e.is_timeout() {
                                handler.on_error(crate::error::Error::new("An error occurred due to timing out...", 1 as u16)).await;
                            }
                            else if e.is_connect() {
                                handler.on_error(crate::error::Error::new("An error occurred when trying to connect...", 2 as u16)).await;
                            }
                            else if e.is_status() {
                                handler.on_error(crate::error::Error::new("Status returned as an Error...", 3 as u16)).await;
                            }
                            else if e.is_redirect() {
                                handler.on_error(crate::error::Error::new("An error occurred due to an attempted redirect...", 4 as u16)).await;
                            }
                            else if e.is_request() {
                                handler.on_error(crate::error::Error::new("An error occurred due to the request...", 5 as u16)).await;
                            }
                            else if e.is_body() {
                                handler.on_error(crate::error::Error::new("An error occurred with the request or response body...", 6 as u16)).await;
                            }
                            else if e.is_builder() {
                                handler.on_error(crate::error::Error::new("An error occurred with the type builder...", 7 as u16)).await;
                            }
                            else {
                                handler.on_error(crate::error::Error::new("An unknown error occurred with the request...", 8 as u16)).await;
                            }
                        }
                    }
                }).await;

                match result {
                    Ok(()) => {},
                    Err(e) => {
                        if e.is_cancelled() {
                            local_client.clone().event_handler.unwrap().on_error(crate::error::Error::new("A Tokio error occurred which resulted in a check being cancelled...", 9)).await;
                        }
                        else if e.is_panic() {
                            local_client.clone().event_handler.unwrap().on_error(crate::error::Error::new("An error occurred causing the Tokio task to panic...", 10)).await;
                        }
                        else {
                            local_client.clone().event_handler.unwrap().on_error(crate::error::Error::new("An unknown Tokio Error Occurred...", 11)).await;
                        }
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
            }
        };
        Ok(())
    }
}