use std::str;
use std::str::FromStr;

use nom::digit;
use nom::types::CompleteStr;
use reqwest::Client;
use select::document::Document;
use select::predicate::{Attr, Class, Name};

use telegram;
use types::Result;

named!(loc_parser(CompleteStr) -> CompleteStr,
   do_parse!(
       many0!(is_not!("(")) >>
       char!('(')           >>
       loc: is_not!(")")    >>
       char!(')')           >>
       (loc)
    )
);

fn get_loc(input: &str) -> Result<String> {
    match loc_parser(CompleteStr(input)) {
        Ok((_, parsed)) => Ok(parsed.to_string()),
        Err(_) => Err("couldn't parse location".to_string())?,
    }
}

named!(
    price_parser(CompleteStr) -> u32,
    map_res!(preceded!(char!('$'), digit), |x: CompleteStr| {FromStr::from_str(&x)})
);

fn get_price(input: &str) -> Result<u32> {
    match price_parser(CompleteStr(input)) {
        Ok((_, parsed)) => Ok(parsed),
        Err(_) => Err("couldn't parse price".to_string())?,
    }
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
    pub fn from_url(url: &str, http_client: &Client) -> Result<Listing> {
        let resp = http_client.get(url).send()?;
        let doc = Document::from_read(resp)?;

        let raw_price = doc
            .find(Class("price"))
            .next()
            .ok_or("price not found")?
            .text();
        let price = get_price(&raw_price)?;

        let title = doc
            .find(Attr("id", "titletextonly"))
            .next()
            .ok_or("name not found")?
            .text();

        let raw_loc = doc
            .find(Name("small"))
            .next()
            .ok_or("location not found")?
            .text();
        let location = get_loc(&raw_loc)?;

        let map = doc.find(Attr("id", "map")).next().ok_or("map not found")?;
        let lat = map
            .attr("data-latitude")
            .ok_or("latitude not found")?
            .parse()?;
        let lon = map
            .attr("data-longitude")
            .ok_or("longitude not found")?
            .parse()?;

        Ok(Listing {
            url: url.to_string(),
            price: price,
            title: title,
            location: location,
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

    pub fn pprint(&self) -> String {
        format!(
            "${price} - {title}\nLocated in {location}\n{url}",
            price = self.price,
            title = self.title,
            location = self.location,
            url = self.url,
        )
    }
}
