use std::sync::Arc;
use std::time::Duration;
use std::num::NonZeroU32;
use std::net::SocketAddr;
use librqbit::{Session, AddTorrent};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::models::*;

#[napi]
pub struct RqbitSession {
    pub(crate) inner: Arc<Session>,
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
            if let Some(enable_upnp) = opts.enable_upnp {
                rqbit_opts.enable_upnp_port_forwarding = enable_upnp;
            }
            if let Some(port) = opts.listen_port {
                rqbit_opts.listen_port_range = Some(port..port + 1);
            }
            if let Some(limit) = opts.concurrent_init_limit {
                rqbit_opts.concurrent_init_limit = Some(limit as usize);
            }
            if let Some(fastresume) = opts.fastresume {
                rqbit_opts.fastresume = fastresume;
            }
            
            if opts.peer_connect_timeout_ms.is_some() || opts.peer_read_write_timeout_ms.is_some() {
                let mut peer_opts = librqbit::PeerConnectionOptions::default();
                if let Some(ms) = opts.peer_connect_timeout_ms {
                    peer_opts.connect_timeout = Some(Duration::from_millis(ms as u64));
                }
                if let Some(ms) = opts.peer_read_write_timeout_ms {
                    peer_opts.read_write_timeout = Some(Duration::from_millis(ms as u64));
                }
                rqbit_opts.peer_opts = Some(peer_opts);
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
        let initial_peers = options.initial_peers.as_ref().map(|peers| {
            peers.iter()
                .filter_map(|p| p.parse::<SocketAddr>().ok())
                .collect::<Vec<_>>()
        });

        let rqbit_options = librqbit::AddTorrentOptions {
            output_folder: options.output_folder.clone(),
            overwrite: options.overwrite.unwrap_or(true),
            paused: options.paused.unwrap_or(false),
            initial_peers,
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
        let initial_peers = options.initial_peers.as_ref().map(|peers| {
            peers.iter()
                .filter_map(|p| p.parse::<SocketAddr>().ok())
                .collect::<Vec<_>>()
        });

        let rqbit_options = librqbit::AddTorrentOptions {
            output_folder: options.output_folder.clone(),
            overwrite: options.overwrite.unwrap_or(true),
            paused: options.paused.unwrap_or(false),
            initial_peers,
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

    #[napi]
    pub fn update_limits(&self, download_bps: Option<u32>, upload_bps: Option<u32>) {
        self.inner.ratelimits.set_download_bps(download_bps.and_then(NonZeroU32::new));
        self.inner.ratelimits.set_upload_bps(upload_bps.and_then(NonZeroU32::new));
    }
}
