module.exports = {
    apps: [
      {
        name: 'l2-micro-service',
        script: 'target/release/l2-micro-service',
        env: {
          ARBITRUM_SEPOLIA_RPC_URL:'https://arbitrum-sepolia.publicnode.com',
          ARBITRUM_MAINNET_RPC_URL:'https://arbitrum-one.publicnode.com',
          DB_URL:'postgresql://postgres:password@localhost:5432/l2indexer',
        }
      },
      {
        name: 'monitor-arbitrum-sepolia',
        script: 'target/release/monitor-events',
        env: {
          CHAIN_TYPE: 'sepolia',
          CHAIN_NAME: 'arbitrum',
          RPC_URL: 'https://ethereum-sepolia.publicnode.com',
          DB_URL:'postgresql://postgres:password@localhost:5432/l2indexer',
        }
      },
      {
        name: 'monitor-base-sepolia',
        script: 'target/release/monitor-events',
        env: {
          CHAIN_TYPE: 'sepolia',
          CHAIN_NAME: 'base',
          RPC_URL: 'https://ethereum-sepolia.publicnode.com',
          DB_URL:'postgresql://postgres:password@localhost:5432/l2indexer',
        }
      },
      {
        name: 'monitor-optimism-sepolia',
        script: 'target/release/monitor-events',
        env: {
          CHAIN_TYPE: 'sepolia',
          CHAIN_NAME: 'optimism',
          RPC_URL: 'https://ethereum-sepolia.publicnode.com',  
          DB_URL:'postgresql://postgres:password@localhost:5432/l2indexer',
        }
      },
      // TODO: Zora docs haven't update sepolia `L2OutputOracle` address
      // {
      //   name: 'monitor-zora-sepolia',
      //   script: 'target/release/monitor-events',
      //   env: {
      //     CHAIN_TYPE: 'sepolia',
      //     CHAIN_NAME: 'zora',
      //     RPC_URL: 'https://ethereum-sepolia.publicnode.com'
      //   }
      // },
     
      {
        name: 'monitor-base-mainnet',
        script: 'target/release/monitor-events',
        env: {
          CHAIN_TYPE: 'mainnet',
          CHAIN_NAME: 'base',
          RPC_URL: 'https://ethereum.publicnode.com',
          DB_URL:'postgresql://postgres:password@localhost:5432/l2indexer',
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
          RPC_URL: 'https://ethereum.publicnode.com',
          DB_URL:'postgresql://postgres:password@localhost:5432/l2indexer',
        }
      },
      {
        name: 'monitor-arbitrum-mainnet',
        script: 'target/release/monitor-events',
        env: {
          CHAIN_TYPE: 'mainnet',
          CHAIN_NAME: 'arbitrum',
          RPC_URL: 'https://ethereum.publicnode.com',
          DB_URL:'postgresql://postgres:password@localhost:5432/l2indexer',
        }
      }
    ]
  };
  