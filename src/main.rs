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
extern crate crossbeam_channel;
extern crate num_cpus;
extern crate rayon;
extern crate reqwest;
extern crate rusqlite;
extern crate select;
extern crate toml;

use clap::{App, Arg};
use crossbeam_channel as channel;
use rayon::prelude::*;
use select::document::Document;
use select::predicate::Class;

use std::sync::Arc;
use std::thread;

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

    let http_client = Arc::new(reqwest::Client::new());

    // Scrape the given url for listings, filtering out what's been already seen.
    // Then fetch all the listing pages, sending their readers over the channel.

    let resp = http_client.get(&cfg.craigslist.url).send()?;
    let doc = Document::from_read(resp)?;

    let (tx, rx) = channel::unbounded();

    doc.find(Class("hdrlnk"))
        .filter_map(|tag| tag.attr("href"))
        .filter(|url| !store.exists(url))
        .for_each(|url| {
            let http_client = Arc::clone(&http_client);
            let tx = tx.clone();
            let url = url.to_string();

            thread::spawn(move || {
                if let Ok(resp) = http_client.get(&url).send() {
                    tx.send(resp);
                }
            });
        });

    drop(tx);

    // Scrape the web pages for listings.

    let pages: Vec<_> = rx.collect();
    let listings: Vec<_> = pages
        .into_par_iter()
        .filter_map(|r| Listing::from_read(r).ok())
        .collect();

    // Send the listings to telegram, saving the ones that have
    // been succesfully transmitted.

    let tel_client = Arc::new(telegram::Client::new(
        &cfg.telegram.token,
        cfg.telegram.chat_id,
    ));

    let handles: Vec<_> = listings
        .into_iter()
        .map(|listing| {
            let tel_client = Arc::clone(&tel_client);

            thread::spawn(move || {
                if listing.post(&tel_client) {
                    println!("{}", listing.pprint());
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    Ok(())
}
