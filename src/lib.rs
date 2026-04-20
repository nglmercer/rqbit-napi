use std::sync::Arc;
use librqbit::{Session, AddTorrent};
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub struct RqbitSession {
    inner: Arc<Session>,
}

#[napi(object)]
#[derive(Default)]
pub struct RqbitAddTorrentOptions {
    pub output_folder: Option<String>,
    pub overwrite: Option<bool>,
}

#[napi(object)]
#[derive(Default)]
pub struct RqbitSessionOptions {
    pub disable_dht: Option<bool>,
    pub disable_dht_persistence: Option<bool>,
}

#[napi]
impl RqbitSession {
    #[napi(factory)]
    pub async fn create(download_path: String, options: Option<RqbitSessionOptions>) -> Result<Self> {
        let mut rqbit_opts = librqbit::SessionOptions::default();
        if let Some(opts) = options {
            if let Some(disable_dht) = opts.disable_dht {
                rqbit_opts.disable_dht = disable_dht;
            }
            if let Some(disable_dht_persistence) = opts.disable_dht_persistence {
                rqbit_opts.disable_dht_persistence = disable_dht_persistence;
            }
        }
        
        let session = Session::new_with_opts(download_path.into(), rqbit_opts)
            .await
            .map_err(|e| Error::from_reason(format!("Failed to create session: {}", e)))?;
        Ok(RqbitSession {
            inner: session,
        })
    }

    #[napi]
    pub async fn add_torrent(&self, url: String, options: Option<RqbitAddTorrentOptions>) -> Result<u32> {
        let options = options.unwrap_or_default();
        let rqbit_options = librqbit::AddTorrentOptions {
            output_folder: options.output_folder,
            overwrite: options.overwrite.unwrap_or(true),
            ..Default::default()
        };
        let response = self.inner.add_torrent(AddTorrent::from_url(url), Some(rqbit_options))
            .await
            .map_err(|e| Error::from_reason(format!("Failed to add torrent: {}", e)))?;
        
        match response {
            librqbit::AddTorrentResponse::Added(id, _) => Ok(id as u32),
            librqbit::AddTorrentResponse::AlreadyManaged(id, _) => Ok(id as u32),
            _ => Err(Error::from_reason("Unexpected response from add_torrent")),
        }
    }

    #[napi]
    pub async fn add_torrent_buffer(&self, buffer: Buffer, options: Option<RqbitAddTorrentOptions>) -> Result<u32> {
        let options = options.unwrap_or_default();
        let rqbit_options = librqbit::AddTorrentOptions {
            output_folder: options.output_folder,
            overwrite: options.overwrite.unwrap_or(true),
            ..Default::default()
        };
        let bytes = bytes::Bytes::from(buffer.as_ref().to_vec());
        let response = self.inner.add_torrent(AddTorrent::from_bytes(bytes), Some(rqbit_options))
            .await
            .map_err(|e| Error::from_reason(format!("Failed to add torrent from buffer: {}", e)))?;
        
        match response {
            librqbit::AddTorrentResponse::Added(id, _) => Ok(id as u32),
            librqbit::AddTorrentResponse::AlreadyManaged(id, _) => Ok(id as u32),
            _ => Err(Error::from_reason("Unexpected response from add_torrent_buffer")),
        }
    }

    #[napi]
    pub async fn get_torrent_stats(&self, index: u32) -> Result<Option<TorrentStats>> {
        if let Some(handle) = self.inner.get(librqbit::api::TorrentIdOrHash::Id(index as usize)) {
            let stats = handle.stats();
            let (download_speed, upload_speed) = stats.live.as_ref()
                .map(|l| (l.download_speed.mbps, l.upload_speed.mbps))
                .unwrap_or((0.0, 0.0));
            
            Ok(Some(TorrentStats {
                name: handle.name().unwrap_or_default(),
                finished: stats.finished,
                total_bytes: stats.total_bytes as i64,
                downloaded_bytes: stats.progress_bytes as i64,
                uploaded_bytes: stats.uploaded_bytes as i64,
                download_speed,
                upload_speed,
            }))
        } else {
            Ok(None)
        }
    }

    #[napi]
    pub async fn pause_torrent(&self, index: u32) -> Result<bool> {
        if let Some(handle) = self.inner.get(librqbit::api::TorrentIdOrHash::Id(index as usize)) {
            self.inner.pause(&handle).await
                .map_err(|e| Error::from_reason(format!("Failed to pause: {}", e)))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[napi]
    pub async fn start_torrent(&self, index: u32) -> Result<bool> {
        if let Some(handle) = self.inner.get(librqbit::api::TorrentIdOrHash::Id(index as usize)) {
            self.inner.unpause(&handle).await
                .map_err(|e| Error::from_reason(format!("Failed to start: {}", e)))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[napi]
    pub fn list_torrents(&self) -> Vec<u32> {
        self.inner.with_torrents(|torrents| {
            torrents.map(|(id, _)| id as u32).collect()
        })
    }

    #[napi]
    pub async fn wait_until_initialized(&self, index: u32) -> Result<bool> {
        if let Some(handle) = self.inner.get(librqbit::api::TorrentIdOrHash::Id(index as usize)) {
            handle.wait_until_initialized().await
                .map_err(|e| Error::from_reason(format!("Failed to wait: {}", e)))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[napi]
    pub async fn delete_torrent(&self, index: u32, delete_files: bool) -> Result<bool> {
        self.inner.delete(librqbit::api::TorrentIdOrHash::Id(index as usize), delete_files).await
            .map_err(|e| Error::from_reason(format!("Failed to delete: {}", e)))?;
        Ok(true)
    }

    #[napi]
    pub fn get_session_stats(&self) -> RqbitSessionStats {
        let stats = self.inner.stats_snapshot();
        RqbitSessionStats {
            fetched_bytes: stats.fetched_bytes as i64,
            uploaded_bytes: stats.uploaded_bytes as i64,
            download_speed: stats.download_speed.mbps,
            upload_speed: stats.upload_speed.mbps,
            uptime_seconds: stats.uptime_seconds as i64,
        }
    }

    #[napi]
    pub async fn stop(&self) {
        self.inner.stop().await;
    }
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
