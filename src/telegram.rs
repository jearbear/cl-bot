use reqwest;

pub struct Client {
    http_client: reqwest::Client,
    base_url: String,
    chat_id: i64,
}

impl Client {
    pub fn new(token: &str, chat_id: i64) -> Client {
        Client {
            http_client: reqwest::Client::new(),
            base_url: format!("https://api.telegram.org/bot{}/sendMessage", token),
            chat_id: chat_id,
        }
    }

    pub fn send_message(&self, msg: &str, notify: bool) -> bool {
        self.http_client
            .post(&self.base_url)
            .form(&[
                ("chat_id", self.chat_id.to_string().as_str()),
                ("parse_mode", "Markdown"),
                ("text", msg),
                (
                    "disable_notification",
                    if notify { "false" } else { "true" },
                ),
            ])
            .send()
            .is_ok()
    }
}
