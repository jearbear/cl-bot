use std::fs::File;
use std::io::Read;

use clap::{App, AppSettings, Arg};
use serde_derive::Deserialize;
use toml;

use crate::store::Store;
use crate::types::*;

pub struct Args {
    pub config: Config,
    pub store: Store,
}

impl Args {
    pub fn init() -> Result<Args> {
        let config_path = Arg::with_name("config")
            .help("The location of the config file to read")
            .required(true);
        let store_path = Arg::with_name("store")
            .help("The location of the store used to keep track of seen listings")
            .long("store")
            .takes_value(true);

        let matches = App::new("cl-bot - A utility to help you keep on top of Craigslist listings")
            .arg(&config_path)
            .arg(&store_path)
            .global_setting(AppSettings::DisableVersion)
            .get_matches();

        let config = Config::from_file(matches.value_of("config").unwrap())?;
        let store = matches
            .value_of("store")
            .map(Store::new)
            .unwrap_or_else(Store::new_in_memory)?;

        Ok(Args { config, store })
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub telegram: TelegramConfig,
    #[serde(rename = "searches")]
    pub searches: Vec<SearchConfig>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config> {
        let mut f = File::open(path)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        let cfg: Config = toml::from_str(&contents)?;

        Ok(cfg)
    }
}

#[derive(Deserialize)]
pub struct SearchConfig {
    pub url: String,
}

#[derive(Deserialize)]
pub struct TelegramConfig {
    pub token: String,
    pub chat_id: i64,
}
