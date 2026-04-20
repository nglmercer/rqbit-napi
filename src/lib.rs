use std::sync::Arc;
use librqbit::{Session, AddTorrent, AddTorrentOptions};
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub struct RqbitSession {
    inner: Arc<Session>,
}

#[napi]
impl RqbitSession {
    #[napi(factory)]
    pub async fn create(download_path: String) -> Result<Self> {
        let session = Session::new(download_path.into())
            .await
            .map_err(|e| Error::from_reason(format!("Failed to create session: {}", e)))?;
        Ok(RqbitSession {
            inner: session,
        })
    }

    #[napi]
    pub async fn add_torrent(&self, url: String, output_folder: Option<String>) -> Result<u32> {
        let options = AddTorrentOptions {
            output_folder,
            ..Default::default()
        };
        let response = self.inner.add_torrent(AddTorrent::from_url(url), Some(options))
            .await
            .map_err(|e| Error::from_reason(format!("Failed to add torrent: {}", e)))?;
        
        match response {
            librqbit::AddTorrentResponse::Added(id, _) => Ok(id as u32),
            librqbit::AddTorrentResponse::AlreadyManaged(id, _) => Ok(id as u32),
            _ => Err(Error::from_reason("Unexpected response from add_torrent")),
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
