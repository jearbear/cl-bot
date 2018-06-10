use std::fs::File;
use std::io::prelude::*;

use toml;

use types::Result;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub craigslist: CraigslistConfig,
    pub telegram: TelegramConfig,
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

#[derive(Deserialize, Debug)]
pub struct CraigslistConfig {
    pub url: String,
    pub limit: usize,
}

#[derive(Deserialize, Debug)]
pub struct TelegramConfig {
    pub token: String,
    pub chat_id: u32,
}
