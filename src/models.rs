use napi_derive::napi;

#[napi(object)]
#[derive(Default)]
pub struct RqbitAddTorrentOptions {
    pub output_folder: Option<String>,
    pub overwrite: Option<bool>,
    pub paused: Option<bool>,
    pub initial_peers: Option<Vec<String>>,
    pub peer_limit: Option<u32>,
}

#[napi(object)]
#[derive(Default)]
pub struct RqbitSessionOptions {
    pub disable_dht: Option<bool>,
    pub disable_dht_persistence: Option<bool>,
    pub enable_upnp: Option<bool>,
    pub listen_port: Option<u16>,
    pub peer_connect_timeout_ms: Option<u32>,
    pub peer_read_write_timeout_ms: Option<u32>,
    pub concurrent_init_limit: Option<u32>,
    pub fastresume: Option<bool>,
    pub peer_limit: Option<u32>,
}

#[napi(object)]
pub struct TorrentStats {
    pub name: String,
    pub finished: bool,
    pub total_bytes: i64,
    pub downloaded_bytes: i64,
    pub uploaded_bytes: i64,
    pub download_speed: f64,
    pub upload_speed: f64,
}

#[napi(object)]
pub struct RqbitSessionStats {
    pub fetched_bytes: i64,
    pub uploaded_bytes: i64,
    pub download_speed: f64,
    pub upload_speed: f64,
    pub uptime_seconds: i64,
}

#[napi(object)]
pub struct PeerStats {
    pub addr: String,
    pub downloaded_bytes: i64,
    pub uploaded_bytes: i64,
    pub download_speed: f64,
    pub upload_speed: f64,
    pub connection_type: String,
    pub state: String,
}
