use std::str::FromStr;

/// A chain name
#[derive(Debug, Clone, Copy)]
pub enum ChainName {
    Arbitrum,
    Base,
    Optimism,
    Zora,
}

impl ToString for ChainName {
    fn to_string(&self) -> String {
        match self {
            ChainName::Arbitrum => "arbitrum".to_string(),
            ChainName::Base => "base".to_string(),
            ChainName::Optimism => "optimism".to_string(),
            ChainName::Zora => "zora".to_string(),
        }
    }
}

impl FromStr for ChainName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "arbitrum" => Ok(ChainName::Arbitrum),
            "base" => Ok(ChainName::Base),
            "optimism" => Ok(ChainName::Optimism),
            "zora" => Ok(ChainName::Zora),
            _ => Err(eyre::eyre!("invalid chain name")),
        }
    }
}

/// A chain name
#[derive(Debug, Clone, Copy)]
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
        let parts: Vec<&str> = s.split('_').collect();
        let chain_name = ChainName::from_str(parts[0])?;
        let chain_type = ChainType::from_str(parts[1])?;
        Ok(Network {
            chain_name,
            chain_type,
        })
    }
}
