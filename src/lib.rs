use napi_derive::napi;
use tracing_subscriber::EnvFilter;

mod models;
mod session;

pub use models::*;
pub use session::*;

#[napi]
pub fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
}
