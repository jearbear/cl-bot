extern crate env_logger;
extern crate failure;
extern crate log;
extern crate nom;
extern crate rayon;
extern crate reqwest;
extern crate rusqlite;
extern crate select;
extern crate serde_derive;
extern crate structopt;
extern crate toml;

mod config;
mod listing;
mod store;
mod telegram;
mod types;

use log::info;
use rayon::prelude::*;
use select::document::Document;
use select::predicate::{Class, Or};
use structopt::StructOpt;

use config::{Config, Opt};
use listing::Listing;
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

    let mut num_posted = 0;

    for search in &cfg.searches {
        info!("Getting listings for `{}`...", search.url);
        let root_page = http_client.get(&search.url).send()?;

        num_posted += Document::from_read(root_page)?
            .find(Or(Class("hdrlnk"), Class("bantext")))
            .take_while(|tag| tag.name() == Some("a"))
            .flat_map(|tag| tag.attr("href"))
            .collect::<Vec<_>>()
            .into_par_iter()
            .filter(|url| !store.exists(url))
            .flat_map(|url| http_client.get(url).send())
            .flat_map(Listing::from_read)
            .inspect(|listing| info!("Saving and posting new listing: {}", listing.title))
            .filter(|listing| listing.post(&tel_client))
            .flat_map(|listing| store.save(&listing.url))
            .count();
    }

    info!(
        "Found {} listings across {} searches",
        num_posted,
        cfg.searches.len()
    );

    if num_posted > 0 {
        tel_client.send_message(&format!("Found {} listings!", num_posted), true);
    }

    Ok(())
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Off)
        .filter_module("cl_bot", log::LevelFilter::Info)
        .default_format_module_path(false)
        .default_format_timestamp(false)
        .init();

    let args = Opt::from_args();

    if let Err(err) = do_main(args) {
        eprintln!("Error: {}", err);
        for cause in err.iter_chain().skip(1) {
            eprintln!("Caused by: {}", cause);
        }
        std::process::exit(1);
    }
}
