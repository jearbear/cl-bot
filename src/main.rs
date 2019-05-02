mod arg;
mod error;
mod listing;
mod store;
mod telegram;

use rayon::prelude::*;
use select::document::Document;
use select::predicate::{Class, Or};

use crate::arg::Args;
use crate::error::Result;
use crate::listing::Listing;

fn do_main(
    Args {
        url,
        store,
        telegram,
    }: Args
) -> Result<()> {
    let http_client = reqwest::Client::new();

    println!("Getting listings for `{}`...", &url);
    let root_page = http_client.get(&url).send()?;

    let num_posted = Document::from_read(root_page)?
        .find(Or(Class("hdrlnk"), Class("bantext")))
        .take_while(|tag| tag.name() == Some("a"))
        .flat_map(|tag| tag.attr("href"))
        .collect::<Vec<_>>()
        .into_par_iter()
        .filter(|url| match &store {
            Some(store) => !store.exists(url),
            None => true,
        })
        .flat_map(|url| http_client.get(url).send())
        .flat_map(Listing::from_read)
        .inspect(|listing| println!("Found new listing: {}", listing.title))
        .filter(|listing| match &telegram {
            Some(tel) => tel.post(&listing),
            None => true,
        })
        .flat_map(|listing| match &store {
            Some(store) => store.save(&listing.url),
            None => Ok(()),
        })
        .count();

    if num_posted > 0 {
        println!("Found {} listings", num_posted);
    }

    Ok(())
}

fn main() {
    if let Err(err) = Args::init().map(do_main) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
