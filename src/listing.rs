use std::io::Read;

use lazy_static::lazy_static;
use regex::Regex;
use select::document::Document;
use select::predicate::{Attr, Class, Name};

use crate::telegram;

fn get_loc(input: &str) -> Option<&str> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\((.*)\)").unwrap();
    }

    Some(RE.captures(input)?.get(1)?.as_str())
}

fn get_price(input: &str) -> Option<u32> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\$(\d+)").unwrap();
    }

    Some(RE.captures(input)?.get(1)?.as_str().parse().ok()?)
}

#[derive(Debug)]
pub struct Listing {
    pub url: String,
    pub price: u32,
    pub title: String,
    pub location: String,
    pub geo: (f32, f32),
}

impl Listing {
    pub fn from_read<R: Read>(reader: R) -> Option<Listing> {
        let doc = Document::from_read(reader).ok()?;

        let url = doc.find(Attr("rel", "canonical")).next()?.attr("href")?;

        let raw_price = doc.find(Class("price")).next()?.text();
        let price = get_price(&raw_price)?;

        let title = doc.find(Attr("id", "titletextonly")).next()?.text();

        let raw_loc = doc.find(Name("small")).next()?.text();
        let location = get_loc(&raw_loc)?;

        let map = doc.find(Attr("id", "map")).next()?;
        let lat = map.attr("data-latitude")?.parse().ok()?;
        let lon = map.attr("data-longitude")?.parse().ok()?;

        Some(Listing {
            url: url.to_owned(),
            price,
            title,
            location: location.to_owned(),
            geo: (lat, lon),
        })
    }

    pub fn post(&self, client: &telegram::Client) -> bool {
        client.send_message(&format!(
            "*${price}* - [{title}]({url})\nLocated in *{location}*",
            price = self.price,
            title = self.title,
            url = self.url,
            location = self.location
        ))
    }
}
