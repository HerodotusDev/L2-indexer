module.exports = {
    apps: [
      {
        name: 'monitor-base-goerli',
        script: 'target/release/monitor-events',
        env: {
          TYPE: 'opstack',
          NETWORK: 'base_goerli',
          RPC_URL: 'https://ethereum-goerli.publicnode.com'
        }
      },
      {
        name: 'monitor-zora-goerli',
        script: 'target/release/monitor-events',
        env: {
          TYPE: 'opstack',
          NETWORK: 'zora_goerli',
          RPC_URL: 'https://ethereum-goerli.publicnode.com'
        }
      },
      {
        name: 'monitor-optimism-goerli',
        script: 'target/release/monitor-events',
        env: {
          TYPE: 'opstack',
          NETWORK: 'optimism_goerli',
          RPC_URL: 'https://ethereum-goerli.publicnode.com'
        }
      },
      {
        name: 'monitor-base-mainnet',
        script: 'target/release/monitor-events',
        env: {
          TYPE: 'opstack',
          NETWORK: 'base_mainnet',
          RPC_URL: 'https://ethereum.publicnode.com'
        }
      },
      {
        name: 'monitor-optimism-mainnet',
        script: 'target/release/monitor-events',
        env: {
          TYPE: 'opstack',
          NETWORK: 'optimism_mainnet',
          RPC_URL: 'https://ethereum.publicnode.com'
        }
      },
      {
        name: 'monitor-zora-mainnet',
        script: 'target/release/monitor-events',
        env: {
          TYPE: 'opstack',
          NETWORK: 'zora_mainnet',
          RPC_URL: 'https://ethereum.publicnode.com'
        }
      },
      {
        name: 'monitor-arbitrum-mainnet',
        script: 'target/release/monitor-events',
        env: {
          TYPE: 'arbitrum',
          NETWORK: 'arbitrum_mainnet',
          RPC_URL: 'https://ethereum.publicnode.com'
        }
      },
    ]
  };
  