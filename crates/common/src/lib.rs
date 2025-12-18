use serde::{Deserialize, Deserializer};
use serde_json;
use std::str::FromStr;

/// Custom deserializer that lowercases address strings
fn deserialize_address_lowercase<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.to_lowercase())
}

/// Custom deserializer that lowercases optional address strings
fn deserialize_optional_address_lowercase<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.map(|s| s.to_lowercase()))
}

/// A chain name
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ChainName {
    Arbitrum,
    ApeChain,
    Base,
    Optimism,
    Zora,
    WorldChain,
}

impl ToString for ChainName {
    fn to_string(&self) -> String {
        match self {
            ChainName::Arbitrum => "arbitrum".to_string(),
            ChainName::ApeChain => "ape_chain".to_string(),
            ChainName::Base => "base".to_string(),
            ChainName::Optimism => "optimism".to_string(),
            ChainName::Zora => "zora".to_string(),
            ChainName::WorldChain => "world_chain".to_string(),
        }
    }
}

impl FromStr for ChainName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "arbitrum" => Ok(ChainName::Arbitrum),
            "ape_chain" => Ok(ChainName::ApeChain),
            "base" => Ok(ChainName::Base),
            "optimism" => Ok(ChainName::Optimism),
            "zora" => Ok(ChainName::Zora),
            "world_chain" => Ok(ChainName::WorldChain),
            _ => Err(eyre::eyre!("invalid chain name")),
        }
    }
}

/// A chain name
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ChainType {
    Mainnet,
    Goerli,
    Sepolia,
}

impl ToString for ChainType {
    fn to_string(&self) -> String {
        match self {
            ChainType::Mainnet => "mainnet".to_string(),
            ChainType::Goerli => "goerli".to_string(),
            ChainType::Sepolia => "sepolia".to_string(),
        }
    }
}

impl FromStr for ChainType {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mainnet" => Ok(ChainType::Mainnet),
            "goerli" => Ok(ChainType::Goerli),
            "sepolia" => Ok(ChainType::Sepolia),
            _ => Err(eyre::eyre!("invalid chain type")),
        }
    }
}

pub struct Network {
    pub chain_name: ChainName,
    pub chain_type: ChainType,
}

impl ToString for Network {
    fn to_string(&self) -> String {
        format!(
            "{}_{}",
            self.chain_name.to_string(),
            self.chain_type.to_string()
        )
    }
}

impl FromStr for Network {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.rsplitn(2, '_').collect();
        let chain_name = ChainName::from_str(parts[1])?;
        let chain_type = ChainType::from_str(parts[0])?;
        Ok(Network {
            chain_name,
            chain_type,
        })
    }
}

/// A struct that represents the Networks struct in the JSON file
#[derive(Debug, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    #[serde(deserialize_with = "deserialize_address_lowercase")]
    pub l1_contract: String,
    pub l1_contract_deployment_block: u64,
    pub block_delay: u64,
    pub poll_period_sec: u64,
    pub batch_size: Option<u64>,
    #[serde(deserialize_with = "deserialize_optional_address_lowercase")]
    pub dispute_game_factory_l1_contract: Option<String>,
    pub l1_dispute_game_contract_deployment_block: Option<u64>,
    pub transition_to_dispute_game_system_block: Option<u64>,
    pub transition_to_dispute_game_system_l2_block: Option<u64>,
    #[serde(deserialize_with = "deserialize_optional_address_lowercase")]
    pub trusted_proposer_address: Option<String>,
}

/// A builder that gets config from embedded JSON and returns NetworkConfig.
/// Parameters:
/// * chain_type: The chain type (mainnet, sepolia, goerli)
/// * chain_name: The chain name (arbitrum, optimism, base, etc.)
/// Returns:
/// * NetworkConfig struct that contains all the network config data
pub fn get_network_config(chain_type: ChainType, chain_name: ChainName) -> NetworkConfig {
    let network = Network {
        chain_name,
        chain_type,
    };
    let config_json = match network.to_string().to_lowercase().as_str() {
        "arbitrum_mainnet" => include_str!("../../monitor_events/networks/arbitrum_mainnet.json"),
        "arbitrum_sepolia" => include_str!("../../monitor_events/networks/arbitrum_sepolia.json"),
        "ape_chain_mainnet" => include_str!("../../monitor_events/networks/ape_chain_mainnet.json"),
        "ape_chain_sepolia" => include_str!("../../monitor_events/networks/ape_chain_sepolia.json"),
        "base_mainnet" => include_str!("../../monitor_events/networks/base_mainnet.json"),
        "base_sepolia" => include_str!("../../monitor_events/networks/base_sepolia.json"),
        "optimism_mainnet" => include_str!("../../monitor_events/networks/optimism_mainnet.json"),
        "optimism_sepolia" => include_str!("../../monitor_events/networks/optimism_sepolia.json"),
        "world_chain_mainnet" => {
            include_str!("../../monitor_events/networks/world_chain_mainnet.json")
        }
        "world_chain_sepolia" => {
            include_str!("../../monitor_events/networks/world_chain_sepolia.json")
        }
        "zora_mainnet" => include_str!("../../monitor_events/networks/zora_mainnet.json"),
        "zora_sepolia" => include_str!("../../monitor_events/networks/zora_sepolia.json"),
        _ => panic!("Unsupported network: {}", network.to_string()),
    };

    serde_json::from_str(config_json).unwrap()
}

pub fn create_network_from_strings(
    chain_name_str: &str,
    chain_type_str: &str,
) -> Result<Network, eyre::Error> {
    let chain_name = ChainName::from_str(chain_name_str)?;
    let chain_type = ChainType::from_str(chain_type_str)?;
    Ok(Network {
        chain_name,
        chain_type,
    })
}
