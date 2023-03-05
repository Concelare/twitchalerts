use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Duration, Utc};
use surrealdb::engine::local::{ Db };
use surrealdb::{ Surreal };
use serde::{Deserialize, Serialize};

use crate::traits::EventHandler;

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Streamer {
    pub id: String,
    pub name: String,
    pub alerts: bool,
    pub last_streamed: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct StreamsRes {
    pub data: Vec<StreamData>,
    pub pagination: Pagination
}

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

#[derive(Serialize, Deserialize)]
pub struct Pagination {
    pub cursor: String
}


#[derive(Clone)]
pub struct Client {
    pub client_id: String,
    pub token: String,
    status: bool,
    event_handler: Option<Arc<dyn EventHandler>>,
    database: Option<Surreal<Db>>

}

impl Client {
    pub fn new(client_id: &str, token: &str) -> Client {
        Client {
            client_id: client_id.to_string(),
            token: token.to_string(),
            status: false,
            event_handler: None,
            database: None,
        }
    }

    pub fn event_handler<H: EventHandler + 'static>(mut self, event_handler: H) -> Self {
        self.event_handler = Some(Arc::new(event_handler));

        self
    }

    pub fn database(mut self, database: Surreal<Db>) -> Self {
        self.database = Some(database);

        self
    }

    pub async fn run(self) -> Result<(), ()> {
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
            tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

            let mut res = local_client.database.as_ref().expect("Error Occurred").query("SELECT * FROM streamers WHERE alerts = true").await.expect("Error Occurred");

            let streamers: Vec<Streamer> = res.take(0).expect("Error Occurred");

            if streamers.is_empty() {
                println!("Ran");
                running = false;
            }

            for streamer in streamers {
                if let Some(time) = recent.get(streamer.name.as_str()) {
                    let difference: Duration = Utc::now() - *time;
                    if 60 > difference.num_seconds() {
                        continue;
                    }
                    else {
                        running = false;
                        recent.remove(streamer.name.as_str());
                    }
                }

                recent.insert(streamer.name.clone(), Utc::now());

                let client_cloned: Client = self.clone();
                let handler = match self.event_handler.clone() {
                    Some(evh) => evh,
                    _ => {panic!("No Event Handler Found");}
                };

                tokio::spawn(async move {
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
                            handler.on_error(e.to_string());
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        }
                    }


                }).await.expect("Error Occurred");
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        };
        Ok(())
    }
}