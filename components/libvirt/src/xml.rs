use serde::{Deserialize, Serialize};

pub mod domain;
pub mod network;

pub use domain::*;
pub use network::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OnOff {
    On,
    Off,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum YesNo {
    Yes,
    No,
}
