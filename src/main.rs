mod config;
mod listing;
mod store;
mod telegram;
mod types;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate nom;
extern crate clap;
extern crate reqwest;
extern crate rusqlite;
extern crate select;
extern crate toml;

use clap::{App, Arg};
use select::document::Document;
use select::predicate::Class;

use config::Config;
use listing::Listing;
use store::Store;
use types::Result;

static INFO: &str = "cl-bot - A handy script to help you keep on top of Craigslist listings";

fn main() -> Result<()> {
    let matches = App::new(INFO)
        .arg(
            Arg::with_name("CONFIG")
                .help("The location of the config file to read")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("STORE")
                .help("The location of the store used to keep track of seen listings")
                .long("store")
                .takes_value(true),
        )
        .get_matches();

    let config_path = matches.value_of("CONFIG").unwrap();
    let cfg = Config::from_file(&config_path)?;

    let store = match matches.value_of("STORE") {
        Some(store_path) => Store::new(store_path)?,
        None => Store::new_in_memory()?,
    };

    let http_client = reqwest::Client::new();

    // Obtain the listings for the most recent listings.
    // Filter out what's already been seen.

    let resp = http_client.get(&cfg.craigslist.url).send()?;
    let doc = Document::from_read(resp)?;

    let listings: Vec<_> = doc.find(Class("hdrlnk"))
        .filter_map(|tag| tag.attr("href"))
        .filter(|url| !store.exists(url))
        .filter_map(|url| http_client.get(url).send().ok())
        .filter_map(move |reader| Listing::from_read(reader).ok())
        .collect();

    // Save and post the listings.

    let tel_client = telegram::Client::new(&cfg.telegram.token, cfg.telegram.chat_id);

    let num_posted = listings
        .iter()
        .filter(|listing| listing.post(&tel_client))
        .filter_map(|listing| store.save(&listing.url).ok())
        .count();

    tel_client.send_message(&format!("Found {} listings!", num_posted), true);

    Ok(())
}
