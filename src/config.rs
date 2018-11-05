use std::fs::File;
use std::io::Read;

use serde_derive::Deserialize;
use structopt::clap::AppSettings;
use structopt::StructOpt;
use toml;

use types::*;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "cl-bot - A handy utility to help you keep on top of Craigslist listings",
    about = "",
    author = "",
    version = "",
    raw(global_setting = "AppSettings::DisableVersion")
)]
pub struct Opt {
    #[structopt(
        name = "config",
        help = "The location of the config file to read"
    )]
    pub config: String,

    #[structopt(
        name = "store",
        help = "The location of the store used to keep track of seen listings",
        long = "store"
    )]
    pub store: Option<String>,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct SearchConfig {
    pub url: String,
    // pub limit: usize,
}

#[derive(Deserialize, Debug)]
pub struct TelegramConfig {
    pub token: String,
    pub chat_id: i64,
}
