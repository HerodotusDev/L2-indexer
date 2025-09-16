use config::{Config, File, FileFormat};
use serde::Deserialize;
use std::str::FromStr;

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
    pub l1_contract: String,
    pub l1_contract_deployment_block: u64,
    pub block_delay: u64,
    pub poll_period_sec: u64,
    pub batch_size: Option<u64>,
    pub dispute_game_factory_l1_contract: Option<String>,
    pub l1_dispute_game_contract_deployment_block: Option<u64>,
    pub transition_to_dispute_game_system_block: Option<u64>,
    pub transition_to_dispute_game_system_l2_block: Option<u64>,
    pub trusted_proposer_address: Option<String>,
}

/// A builder that gets config from JSON and returns Config.
/// Parameters:
/// * network_config: The name of the network want to get from JSON
/// Returns:
/// * Networks struct that contains all the network config data
pub fn get_network_config(chain_type: ChainType, chain_name: ChainName) -> NetworkConfig {
    let network = Network {
        chain_name,
        chain_type,
    };
    let config_name = format!(
        "crates/monitor_events/networks/{}",
        network.to_string().to_lowercase()
    );
    let config = Config::builder()
        .add_source(File::new(&config_name, FileFormat::Json))
        .build()
        .unwrap();
    config.try_deserialize().unwrap()
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
