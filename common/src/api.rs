pub mod compute;
pub mod network;
pub mod storage;

pub trait ApiBase: Send + Sync + 'static {}
