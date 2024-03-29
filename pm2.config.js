module.exports = {
    apps: [
      {
        name: 'monitor-arbitrum-sepolia',
        script: 'target/release/monitor-events',
        env: {
          CHAIN_TYPE: 'sepolia',
          CHAIN_NAME: 'arbitrum',
          RPC_URL: 'https://ethereum-sepolia.publicnode.com'
        }
      },
      {
        name: 'monitor-base-sepolia',
        script: 'target/release/monitor-events',
        env: {
          CHAIN_TYPE: 'sepolia',
          CHAIN_NAME: 'base',
          RPC_URL: 'https://ethereum-sepolia.publicnode.com'
        }
      },
      {
        name: 'monitor-base-goerli',
        script: 'target/release/monitor-events',
        env: {
          CHAIN_TYPE: 'goerli',
          CHAIN_NAME: 'base',
          RPC_URL: 'https://ethereum-goerli.publicnode.com'
        }
      },
      {
        name: 'monitor-zora-goerli',
        script: 'target/release/monitor-events',
        env: {
          CHAIN_TYPE: 'goerli',
          CHAIN_NAME: 'zora',
          RPC_URL: 'https://ethereum-goerli.publicnode.com'
        }
      },
      {
        name: 'monitor-optimism-goerli',
        script: 'target/release/monitor-events',
        env: {
          CHAIN_TYPE: 'goerli',
          CHAIN_NAME: 'optimism',
          RPC_URL: 'https://ethereum-goerli.publicnode.com'
        }
      },
      {
        name: 'monitor-base-mainnet',
        script: 'target/release/monitor-events',
        env: {
          CHAIN_TYPE: 'mainnet',
          CHAIN_NAME: 'base',
          RPC_URL: 'https://ethereum.publicnode.com'
        }
      },
      {
        name: 'monitor-optimism-mainnet',
        script: 'target/release/monitor-events',
        env: {
          CHAIN_TYPE: 'mainnet',
          CHAIN_NAME: 'optimism',
          RPC_URL: 'https://ethereum.publicnode.com'
        }
      },
      {
        name: 'monitor-zora-mainnet',
        script: 'target/release/monitor-events',
        env: {
          CHAIN_TYPE: 'mainnet',
          CHAIN_NAME: 'zora',
          RPC_URL: 'https://ethereum.publicnode.com'
        }
      },
      {
        name: 'monitor-arbitrum-mainnet',
        script: 'target/release/monitor-events',
        env: {
          CHAIN_TYPE: 'mainnet',
          CHAIN_NAME: 'arbitrum',
          RPC_URL: 'https://ethereum.publicnode.com'
        }
      }
    ]
  };
  