use std::fmt::Display;

use ckb_cinnabar_calculator::re_exports::eyre;
use reqwest::Url;

#[derive(PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet,
    Custom(Url),
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Mainnet => write!(f, "mainnet"),
            Network::Testnet => write!(f, "testnet"),
            Network::Custom(url) => write!(f, "{}", url),
        }
    }
}

impl TryFrom<String> for Network {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "mainnet" => Ok(Network::Mainnet),
            "testnet" => Ok(Network::Testnet),
            _ => Ok(Network::Custom(value.parse()?)),
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum TypeIdMode {
    Keep,
    Remove,
    New,
}

impl TryFrom<String> for TypeIdMode {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "keep" => Ok(TypeIdMode::Keep),
            "remove" => Ok(TypeIdMode::Remove),
            "new" => Ok(TypeIdMode::New),
            _ => Err(eyre::eyre!("invalid type_id_mode")),
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum ListMode {
    All,
    Deployed,
    Consumed,
}

impl TryFrom<String> for ListMode {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "all" => Ok(ListMode::All),
            "deployed" => Ok(ListMode::Deployed),
            "consumed" => Ok(ListMode::Consumed),
            _ => Err(eyre::eyre!("invalid list mode")),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct DeploymentRecord {
    pub name: String,
    pub date: String,
    pub operation: String,
    pub version: String,
    pub tx_hash: String,
    pub out_index: usize,
    pub data_hash: Option<String>,
    pub occupied_capacity: u64,
    pub payer_address: String,
    pub owner_address: Option<String>,
    pub type_id: Option<String>,
    // This field is not required, so you can edit in your <contract>.json file to add comment for cooperations
    #[serde(default)]
    pub comment: Option<String>,
}

impl DeploymentRecord {
    pub fn contract_owner_address(&self) -> String {
        self.owner_address
            .clone()
            .unwrap_or_else(|| self.payer_address.clone())
    }
}
