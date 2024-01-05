module.exports = {
    apps: [
      {
        name: 'monitor-base-goerli',
        script: 'target/release/monitor-events',
        env: {
          TYPE: 'opstack',
          NETWORK: 'base_goerli',
          RPC_URL: 'https://eth-goerli.g.alchemy.com/v2/OxCXO750oi6BTN1kndUMScfn6a16gFIm'
        }
      },
      {
        name: 'monitor-zora-goerli',
        script: 'target/release/monitor-events',
        env: {
          TYPE: 'opstack',
          NETWORK: 'zora_goerli',
          RPC_URL: 'https://eth-goerli.g.alchemy.com/v2/OxCXO750oi6BTN1kndUMScfn6a16gFIm'
        }
      },
      {
        name: 'monitor-optimism-goerli',
        script: 'target/release/monitor-events',
        env: {
          TYPE: 'opstack',
          NETWORK: 'optimism_goerli',
          RPC_URL: 'https://eth-goerli.g.alchemy.com/v2/OxCXO750oi6BTN1kndUMScfn6a16gFIm'
        }
      },
      {
        name: 'monitor-base-mainnet',
        script: 'target/release/monitor-events',
        env: {
          TYPE: 'opstack',
          NETWORK: 'base_mainnet',
          RPC_URL: 'https://eth-mainnet.g.alchemy.com/v2/Am-SA6lZl7P1G2NY0D4Gim1pexDl8ghI'
        }
      },
      {
        name: 'monitor-optimism-mainnet',
        script: 'target/release/monitor-events',
        env: {
          TYPE: 'opstack',
          NETWORK: 'optimism_mainnet',
          RPC_URL: 'https://eth-mainnet.g.alchemy.com/v2/Am-SA6lZl7P1G2NY0D4Gim1pexDl8ghI'
        }
      },
      {
        name: 'monitor-zora-mainnet',
        script: 'target/release/monitor-events',
        env: {
          TYPE: 'opstack',
          NETWORK: 'zora_mainnet',
          RPC_URL: 'https://eth-mainnet.g.alchemy.com/v2/Am-SA6lZl7P1G2NY0D4Gim1pexDl8ghI'
        }
      },
      {
        name: 'monitor-arbitrum-mainnet',
        script: 'target/release/monitor-events',
        env: {
          TYPE: 'arbitrum',
          NETWORK: 'arbitrum_mainnet',
          RPC_URL: 'https://eth-mainnet.g.alchemy.com/v2/Am-SA6lZl7P1G2NY0D4Gim1pexDl8ghI'
        }
      },
    ]
  };
  