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
extern crate rayon;
extern crate reqwest;
extern crate rusqlite;
extern crate select;
extern crate toml;

use clap::{App, Arg};
use rayon::prelude::*;
use select::document::Document;
use select::predicate::Class;

use config::Config;
use listing::Listing;
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
                .help("The location of the store used to keep track of seen listings. (default: default.db)")
                .long("store")
                .takes_value(true),
        )
        .get_matches();

    let config_path = matches.value_of("CONFIG").unwrap();
    let cfg = Config::from_file(&config_path)?;

    let store_path = matches.value_of("STORE").unwrap_or("default.db");
    let store = store::Store::new(store_path)?;

    let http_client = reqwest::Client::new();

    // Scrape the given url for listings, filtering out what's been already seen.

    let resp = http_client.get(&cfg.craigslist.url).send()?;
    let doc = Document::from_read(resp)?;

    let urls: Vec<_> = doc
        .find(Class("hdrlnk"))
        .filter_map(|tag| tag.attr("href"))
        .collect();

    let listings: Vec<_> = urls
        .iter()
        .filter(|url| store.save(url).is_ok())
        .filter_map(|url| Listing::from_url(&url, &http_client).ok())
        .take(cfg.craigslist.limit)
        .collect();

    // Post any new listings to telegram.

    if listings.is_empty() {
        println!("No new listings found.");
        return Ok(());
    }

    let tel_client = telegram::Client::new(&cfg.telegram.token, cfg.telegram.chat_id);

    println!("Found {} new listings:\n", listings.len());
    listings.par_iter().for_each(|listing| {
        listing.post(&tel_client);
        println!("{}\n", listing.pprint());
    });

    Ok(())
}
