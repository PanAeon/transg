use serde::de::DeserializeOwned;
use serde_json::json;
use serde_json::Value;
use std::cell::RefCell;
use serde::Deserialize;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub struct TransmissionClient {
    client: reqwest::Client,
    session_id: RefCell<String>,
    url: String,
}

#[derive(Deserialize, Debug)]
pub struct RpcResponse<T> {
    pub arguments: T,
    pub result: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FreeSpace {
    pub path: String,
    #[serde(rename = "size-bytes")]
    pub size_bytes: u64,
//    pub total_size: u64
}

#[derive(Deserialize, Debug, Clone)]
pub struct Stats {
    #[serde(rename = "uploadedBytes")]
    pub upload_bytes: u64,
    #[serde(rename = "downloadedBytes")]
    pub download_bytes: u64,
    #[serde(rename = "filesAdded")]
    pub files_added: u64,
    #[serde(rename = "sessionCount")]
    pub session_count: u64,
    #[serde(rename = "secondsActive")]
    pub seconds_active: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SessionStats {
    #[serde(rename = "activeTorrentCount")]
    pub active_torrent_count: u64,
    #[serde(rename = "downloadSpeed")]
    pub download_speed: u64,
    #[serde(rename = "pausedTorrentCount")]
    pub paused_torrent_count: u64,
    #[serde(rename = "torrentCount")]
    pub torrent_count: u64,
    #[serde(rename = "uploadSpeed")]
    pub upload_speed: u64,
    #[serde(rename = "current-stats")]
    pub current_stats: Stats,
    #[serde(rename = "cumulative-stats")]
    pub cumulative_stats: Stats,
}

#[derive(Deserialize, Debug, Clone)]
pub struct File {
    pub name: String,
    pub length: u64,
    #[serde(rename = "bytesCompleted")]
    pub bytes_completed: u64
}

#[derive(Deserialize, Debug, Clone)]
pub struct FileStats {
  pub wanted: bool,
  pub priority: i8,
  #[serde(rename = "bytesCompleted")]
  pub bytes_completed: u64,
}


#[derive(Deserialize, Debug, Clone)]
pub struct TrackerStats {
  #[serde(rename = "leecherCount")]
  pub leecher_count: i64,
  pub id: u64,
  pub host: String,
  pub scrape: String,
  #[serde(rename = "seederCount")]
  pub seeder_count: i64,
  #[serde(rename = "lastAnnouncePeerCount")]
  pub last_announce_peer_count: u64,
  #[serde(rename = "lastAnnounceResult")]
  pub last_announce_result: String,
  #[serde(rename = "lastAnnounceTime")]
  pub last_announce_time: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Tracker {
 pub id: u64,
 pub announce: String,
 pub scrape: String,
 pub tier: u64
}

#[derive(Deserialize, Debug, Clone)]
pub struct Peer {
  pub address: String,
  #[serde(rename = "clientName")]
  pub client_name: String,
  pub progress: f64,
  #[serde(rename = "rateToClient")]
  pub rate_to_client: u64,
  #[serde(rename = "rateToPeer")]
  pub rate_to_peer: u64,
  #[serde(rename = "flagStr")]
  pub flag_str: String,
}
static TORRENT_DETAILS_FIELDS: &'static[&'static str] = &[
  "id", "name", "eta", "sizeWhenDone", "seederCount", "leecherCount",
  "downloadDir", "comment", "hashString", "rateDownload", "rateUpload",
  "uploadRatio", "seedRatioLimit", "priority", "doneDate", "percentDone",
  "downloadedEver", "uploadedEver", "corruptEver", "status",
  "labels", "pieceCount", "pieces", "files", "fileStats", "priorities",
  "wanted", "peers", "peer", "trackers", "trackerStats"
];
#[derive(Deserialize, Debug, Clone)]
pub struct Torrents {
    pub torrents: Vec<TorrentDetails>
}
#[derive(Deserialize, Debug, Clone)]
pub struct TorrentDetails {
    pub id: u64,
    pub name: String,
    pub eta: i64,
    #[serde(rename = "sizeWhenDone")]
    pub size_when_done: u64,
    #[serde(rename = "seederCount")]
    pub seeder_count: i64,
    #[serde(rename = "leecherCount")]
    pub leecher_count: i64,
    pub status: u64,
    #[serde(rename = "downloadDir")]
    pub download_dir: String,
    #[serde(rename = "comment")]
    pub comment: String,
    #[serde(rename = "hashString")]
    pub hash_string: String,
    #[serde(rename = "rateDownload")]
    pub rate_download: u64,
    #[serde(rename = "rateUpload")]
    pub rate_upload: u64,
    #[serde(rename = "uploadRatio")]
    pub upload_ratio: f64,
    #[serde(rename = "seedRatioLimit")]
    pub seed_ratio_limit: u64,
    #[serde(rename = "priority")]
    pub priority: u64,
    #[serde(rename = "doneDate")]
    pub done_date: u64,
    #[serde(rename = "percentDone")]
    pub percent_complete: f64,
    #[serde(rename = "downloadedEver")]
    pub downloaded_ever: u64,
    #[serde(rename = "uploadedEver")]
    pub uploaded_ever: u64,
    #[serde(rename = "corruptEver")]
    pub corrupt_ever: u64,
    pub labels: Vec<String>,
    #[serde(rename = "pieceCount")]
    pub piece_count: u64,
    pub pieces: String, // base64 encoded bitstring
    pub files: Vec<File>,
    #[serde(rename = "fileStats")]
    pub file_stats: Vec<FileStats>,
    pub priorities: Vec<u8>,
    pub wanted: Vec<u8>,
    pub peers: Vec<Peer>,
    pub trackers: Vec<Tracker>,
    #[serde(rename = "trackerStats")]
    pub tracker_stats: Vec<TrackerStats>,
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

    pub async fn get_session_stats(&self) -> Result<RpcResponse<SessionStats>> {
        Ok(self
            .execute(json!({
                 "method": "session-stats"
            })).await?)
    }

    pub async fn get_free_space(&self, path: &str) -> Result<RpcResponse<FreeSpace>> {
        Ok(self.execute(json!({
                 "method": "free-space",
                 "arguments": {
                     "path": &path
                 }
            })).await?)
    }

    #[allow(dead_code)]
    pub async fn get_torrent_details(&self, ids: Vec<i64> ) -> Result<RpcResponse<Torrents>> {
        self
            .execute(json!({
                 "method": "torrent-get",
                 "arguments": {
                   "ids": &ids,
                   "fields": TORRENT_DETAILS_FIELDS,
                   "format": "objects"
                 }
            }))
            .await
    }

    #[allow(dead_code)]
    pub async fn get_torrents(&self, ids: Vec<i64>, fields: &Vec<&str>) -> Result<Value> {
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

    pub async fn queue_move_top(&self, ids: Vec<i64>) -> Result<Value> {
        Ok(self
            .execute(json!({
                 "method": "queue-move-top",
                 "arguments": {
                   "ids": &ids
                 }
            }))
            .await?)
    }

    pub async fn queue_move_up(&self, ids: Vec<i64>) -> Result<Value> {
        Ok(self
            .execute(json!({
                 "method": "queue-move-top",
                 "arguments": {
                   "ids": &ids
                 }
            }))
            .await?)
    }

    pub async fn queue_move_bottom(&self, ids: Vec<i64>) -> Result<Value> {
        Ok(self
            .execute(json!({
                 "method": "queue-move-bottom",
                 "arguments": {
                   "ids": &ids
                 }
            }))
            .await?)
    }

    pub async fn queue_move_down(&self, ids: Vec<i64>) -> Result<Value> {
        Ok(self
            .execute(json!({
                 "method": "queue-move-down",
                 "arguments": {
                   "ids": &ids
                 }
            }))
            .await?)
    }

    // returnes also removed array of torrent-id numbers of recently-removed torrents.
    pub async fn get_recent_torrents(&self, fields: &Vec<&str>) -> Result<Value> {
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

    pub async fn get_all_torrents(&self, fields: &Vec<&str>) -> Result<Value> {
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

    pub async fn execute<R>(&self, json: Value) -> Result<R> 
    where 
      R: DeserializeOwned + std::fmt::Debug
    {
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
                println!("getting new CSRF token");
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
        let json = response.json().await?;
        //println!("Response body: {:#?}", json);
        serde_json::from_value(json).map_err(From::from)
    }
}
