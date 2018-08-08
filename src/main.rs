#[macro_use]
extern crate failure;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;

extern crate rayon;
extern crate reqwest;
extern crate rusqlite;
extern crate select;
extern crate toml;

mod config;
mod listing;
mod store;
mod telegram;
mod types;

use rayon::prelude::*;
use select::document::Document;
use select::predicate::Class;
use structopt::StructOpt;

use config::{Config, Opt};
use listing::{get_region, Listing};
use store::Store;
use types::*;

fn do_main(args: Opt) -> Result<()> {
    let cfg = Config::from_file(&args.config).context("Error loading config file")?;

    let store = match &args.store {
        Some(store_path) => Store::new(store_path).context("Error initializing store")?,
        None => Store::new_in_memory()?,
    };

    let http_client = reqwest::Client::new();
    let tel_client = telegram::Client::new(&cfg.telegram.token, cfg.telegram.chat_id);

    let root_region = get_region(&cfg.craigslist.url)?;
    let root_page = http_client.get(&cfg.craigslist.url).send()?;

    let num_posted = Document::from_read(root_page)?
        .find(Class("hdrlnk"))
        .flat_map(|tag| tag.attr("href"))
        .collect::<Vec<_>>()
        .into_par_iter()
        .filter(|url| get_region(url).map(|r| root_region == r).unwrap_or(false))
        .filter(|url| !store.exists(url))
        .flat_map(|url| http_client.get(url).send())
        .flat_map(Listing::from_read)
        .filter(|listing| listing.post(&tel_client))
        .flat_map(|listing| store.save(&listing.url))
        .count();

    if num_posted > 0 {
        tel_client.send_message(&format!("Found {} listings!", num_posted), true);
    }

    Ok(())
}

fn main() {
    let args = Opt::from_args();

    if let Err(err) = do_main(args) {
        eprintln!("Error: {}", err);
        for cause in err.iter_chain().skip(1) {
            eprintln!("Caused by: {}", cause);
        }
        std::process::exit(1);
    }
}
