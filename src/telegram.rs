use reqwest;

use crate::error::Result;
use crate::listing::Listing;

pub struct Client {
    http_client: reqwest::Client,
    base_url: String,
    chat_id: String,
}

impl Client {
    pub fn new(token: &str, chat_id: &str) -> Result<Client> {
        let client = Client {
            http_client: reqwest::Client::new(),
            base_url: format!("https://api.telegram.org/bot{}", token),
            chat_id: chat_id.to_string(),
        };

        if client.ping() {
            Ok(client)
        } else {
            Err("Couldn't connect to telegram API".into())
        }
    }

    fn ping(&self) -> bool {
        self.http_client
            .get(&format!("{}/getChat", self.base_url))
            .form(&[("chat_id", &self.chat_id)])
            .send()
            .map(|s| s.status().is_success())
            .unwrap_or(false)
    }

    pub fn post(&self, listing: &Listing) -> bool {
        let msg = &format!(
            "*${price}* - [{title}]({url})\nLocated in *{location}*",
            price = listing.price,
            title = listing.title,
            url = listing.url,
            location = listing.location
        );

        self.http_client
            .post(&format!("{}/sendMessage", self.base_url))
            .form(&[
                ("chat_id", self.chat_id.as_ref()),
                ("parse_mode", "Markdown"),
                ("text", &msg),
                ("disable_notification", "true"),
            ])
            .send()
            .map(|s| s.status().is_success())
            .unwrap_or(false)
    }
}
