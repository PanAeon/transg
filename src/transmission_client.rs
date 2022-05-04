use serde_json::json;
use serde_json::Value;
use std::cell::RefCell;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub struct TransmissionClient {
    client: reqwest::Client,
    session_id: RefCell<String>,
    url: String,
}

// FIXME: how to work with http errors? async errors?
// від заумі інтелігентськой, митця пожалуста спасі, щоб естетичний код продукту розшифрувать могли
// усі
impl TransmissionClient {
    pub fn new(url: &str) -> TransmissionClient {
        TransmissionClient {
            client: reqwest::Client::new(),
            session_id: RefCell::new("".to_string()),
            url: url.to_string(),
        }
    }

    pub async fn get_torrents(&self, ids: Vec<i64>, fields: Vec<&str>) -> Result<Value> {
        Ok(self
            .execute(json!({
                 "method": "torrent-get",
                 "arguments": {
                   "ids": &ids,
                   "fields": &fields,
                   "format": "table"
                 }
            }))
            .await?)
    }

    // returnes also removed array of torrent-id numbers of recently-removed torrents.
    pub async fn get_recent_torrents(&self, fields: Vec<&str>) -> Result<Value> {
        Ok(self
            .execute(json!({
                 "method": "torrent-get",
                 "arguments": {
                   "ids": "recently-active",
                   "fields": &fields,
                   "format": "table"
                 }
            }))
            .await?)
    }

    pub async fn get_all_torrents(&self, fields: Vec<&str>) -> Result<Value> {
        Ok(self
            .execute(json!({
                 "method": "torrent-get",
                 "arguments": {
                   "fields": &fields,
                   "format": "table"
                 }
            }))
            .await?)
    }

    pub async fn execute(&self, json: Value) -> Result<Value> {
        let mut sid = self.session_id.borrow_mut();

        let response = self
            .client
            .post(&self.url)
            .header("X-Transmission-Session-Id", sid.to_string())
            .json(&json)
            .send()
            .await?;

        let response = match response.status() {
            reqwest::StatusCode::CONFLICT => {
                *sid = response
                    .headers()
                    .get("x-transmission-session-id")
                    .expect("server returned no CSRF token.")
                    .to_str()
                    .expect("wrong CSRF token.")
                    .to_string();
                self.client
                    .post(&self.url)
                    .header("X-Transmission-Session-Id", sid.to_string())
                    .json(&json)
                    .send()
                    .await?
            }
            _ => response,
        };
        Ok(response.json().await?)
    }
}
