mod config;
mod listing;
mod store;
mod telegram;
mod types;

use rayon::prelude::*;
use select::document::Document;
use select::predicate::{Class, Or};

use crate::config::Args;
use crate::listing::Listing;
use crate::types::*;

fn do_main(Args { config, store }: Args) -> Result<()> {
    let http_client = reqwest::Client::new();
    let tel_client = telegram::Client::new(&config.telegram.token, config.telegram.chat_id);

    let mut num_posted = 0;

    for search in &config.searches {
        println!("Getting listings for `{}`...", search.url);
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
            .inspect(|listing| println!("Saving and posting new listing: {}", listing.title))
            .filter(|listing| listing.post(&tel_client))
            .flat_map(|listing| store.save(&listing.url))
            .count();
    }

    println!(
        "Found {} listings across {} searches",
        num_posted,
        config.searches.len()
    );

    if num_posted > 0 {
        tel_client.send_message(&format!("Found {} listings!", num_posted));
    }

    Ok(())
}

fn main() {
    if let Err(err) = Args::init().map(do_main) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
