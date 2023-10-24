use std::env;
use serde::{Deserialize, Serialize};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct Config {
    pub streamers: Vec<String>,
    pub delay: Option<u16>,
    pub token: Option<String>,
    pub user_id: Option<String>
}

impl Default for Config {
    fn default() -> Self {
        Config {
            streamers: Vec::new(),
            delay: Some(80u16),
            token: Some(String::new()),
            user_id: Some(String::new())
        }
    }
}

pub(crate) async fn read_config() -> Config {
    if !env::current_exe().unwrap().parent().unwrap().join("/TwitchAlertsConfig.toml").as_path().exists() {
        write_config(Config::default()).await;
    }

    let file = File::open(env::current_exe().unwrap().parent().unwrap().join("/TwitchAlertsConfig.toml").as_path()).await;

    if file.is_err() {
        panic!("An error occurred opening file!")
    }
    let mut c_string = String::new();
    file.unwrap().read_to_string(&mut c_string).await;

    let config = toml::from_str::<Config>(c_string.as_str());

    if config.is_err() {
        panic!("An error occurred deserializing the file!, {0}  {1:?}", config.err().unwrap(), env::current_exe().unwrap().parent().unwrap().join("/TwitchAlertsConfig.toml").as_path())
    }

    return config.unwrap();
}

pub(crate) async fn write_config(config: Config) {

    if !env::current_exe().unwrap().parent().unwrap().join("/TwitchAlertsConfig.toml").exists() {
        let create = File::create(env::current_exe().unwrap().parent().unwrap().join("/TwitchAlertsConfig.toml").as_path()).await;
        if create.is_err() {
            panic!("An error occurred creating the config file!")
        }
    }

    let file = OpenOptions::new()
        .read(true).write(true).open(env::current_exe().unwrap().parent().unwrap().join("/TwitchAlertsConfig.toml").as_path()).await;


    let tconfig = toml::to_string(&config);

    if tconfig.is_err() {
        panic!("An error occurred serializing the file!")
    }

    let write = file.unwrap().write_all(tconfig.unwrap().as_bytes()).await;

    if write.is_err() {
        panic!("An error occurred writing the file!")
    }
}