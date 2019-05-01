use clap::{App, AppSettings, Arg, ArgGroup};

use crate::error::*;
use crate::store::Store;
use crate::telegram;

pub struct Args {
    pub url: String,
    pub store: Option<Store>,
    pub telegram: Option<telegram::Client>,
}

impl Args {
    pub fn init() -> Result<Args> {
        let url = Arg::with_name("url")
            .help("The craigslist url to fetch")
            .long("url")
            .takes_value(true)
            .required(true);

        let store_path = Arg::with_name("store")
            .help("The location of the store used to keep track of seen listings")
            .long("store")
            .takes_value(true);

        let tel_token = Arg::with_name("tel-token")
            .help("Your telegram bot API token")
            .long("tel-token")
            .takes_value(true);

        let tel_chat_id = Arg::with_name("tel-chat-id")
            .help("The ID of the telegram chat to post listings to")
            .long("tel-chat-id")
            .takes_value(true);

        let tel_group = ArgGroup::with_name("tel-group")
            .args(&["tel-token", "tel-chat-id"])
            .requires_all(&["tel-token", "tel-chat-id"])
            .multiple(true);

        let matches = App::new("cl-fetch - A utility to collect craigslist listings")
            .args(&[url, store_path, tel_token, tel_chat_id])
            .group(tel_group)
            .global_setting(AppSettings::DisableVersion)
            .get_matches();

        let store = match matches.value_of("store") {
            Some(store_path) => Some(Store::new(store_path)?),
            None => None,
        };

        let telegram = if matches.is_present("tel-group") {
            Some(telegram::Client::new(
                matches.value_of("tel-token").unwrap(),
                matches.value_of("tel-chat-id").unwrap(),
            )?)
        } else {
            None
        };

        Ok(Args {
            url: matches.value_of("url").unwrap().to_string(),
            store,
            telegram,
        })
    }
}
