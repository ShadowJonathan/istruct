use super::YesNo;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "network")]
pub struct Network {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<Uuid>,

    #[serde(rename = "$attr:ipv6", skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<YesNo>,
    #[serde(
        rename = "$attr:trustGuestRxFilters",
        skip_serializing_if = "Option::is_none"
    )]
    pub trust_guest_rx_filters: Option<YesNo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct NetworkDoc {
    network: Network,
}

impl Network {
    pub fn to_string(self) -> Result<String, xml_serde::Error> {
        xml_serde::to_string(&NetworkDoc { network: self })
    }

    pub fn from_str(s: &str) -> Result<Self, xml_serde::Error> {
        xml_serde::from_str::<NetworkDoc>(s).map(|nd| nd.network)
    }
}
