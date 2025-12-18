-- Migration: 001_initial_schema.sql
-- Description: Initial database schema for L2 indexer
-- Creates all tables for OPStack chains, Arbitrum chains, and fault dispute games

-- ============================================================================
-- OPStack Tables (Optimism, Base, Zora, WorldChain)
-- ============================================================================

-- Optimism Mainnet
CREATE TABLE IF NOT EXISTS optimism_mainnet (
    id                      SERIAL PRIMARY KEY,
    l2_output_root          VARCHAR NOT NULL,
    l2_output_index         INTEGER NOT NULL,
    l2_block_number         INTEGER NOT NULL,
    l1_timestamp            INTEGER NOT NULL,
    l1_transaction_hash     VARCHAR NOT NULL,
    l1_block_number         INTEGER NOT NULL,
    l1_transaction_index    INTEGER NOT NULL,
    l1_block_hash           VARCHAR NOT NULL
);

-- Optimism Sepolia
CREATE TABLE IF NOT EXISTS optimism_sepolia (
    id                      SERIAL PRIMARY KEY,
    l2_output_root          VARCHAR NOT NULL,
    l2_output_index         INTEGER NOT NULL,
    l2_block_number         INTEGER NOT NULL,
    l1_timestamp            INTEGER NOT NULL,
    l1_transaction_hash     VARCHAR NOT NULL,
    l1_block_number         INTEGER NOT NULL,
    l1_transaction_index    INTEGER NOT NULL,
    l1_block_hash           VARCHAR NOT NULL
);

-- Base Mainnet
CREATE TABLE IF NOT EXISTS base_mainnet (
    id                      SERIAL PRIMARY KEY,
    l2_output_root          VARCHAR NOT NULL,
    l2_output_index         INTEGER NOT NULL,
    l2_block_number         INTEGER NOT NULL,
    l1_timestamp            INTEGER NOT NULL,
    l1_transaction_hash     VARCHAR NOT NULL,
    l1_block_number         INTEGER NOT NULL,
    l1_transaction_index    INTEGER NOT NULL,
    l1_block_hash           VARCHAR NOT NULL
);

-- Base Sepolia
CREATE TABLE IF NOT EXISTS base_sepolia (
    id                      SERIAL PRIMARY KEY,
    l2_output_root          VARCHAR NOT NULL,
    l2_output_index         INTEGER NOT NULL,
    l2_block_number         INTEGER NOT NULL,
    l1_timestamp            INTEGER NOT NULL,
    l1_transaction_hash     VARCHAR NOT NULL,
    l1_block_number         INTEGER NOT NULL,
    l1_transaction_index    INTEGER NOT NULL,
    l1_block_hash           VARCHAR NOT NULL
);

-- Zora Mainnet
CREATE TABLE IF NOT EXISTS zora_mainnet (
    id                      SERIAL PRIMARY KEY,
    l2_output_root          VARCHAR NOT NULL,
    l2_output_index         INTEGER NOT NULL,
    l2_block_number         INTEGER NOT NULL,
    l1_timestamp            INTEGER NOT NULL,
    l1_transaction_hash     VARCHAR NOT NULL,
    l1_block_number         INTEGER NOT NULL,
    l1_transaction_index    INTEGER NOT NULL,
    l1_block_hash           VARCHAR NOT NULL
);

-- Zora Sepolia
CREATE TABLE IF NOT EXISTS zora_sepolia (
    id                      SERIAL PRIMARY KEY,
    l2_output_root          VARCHAR NOT NULL,
    l2_output_index         INTEGER NOT NULL,
    l2_block_number         INTEGER NOT NULL,
    l1_timestamp            INTEGER NOT NULL,
    l1_transaction_hash     VARCHAR NOT NULL,
    l1_block_number         INTEGER NOT NULL,
    l1_transaction_index    INTEGER NOT NULL,
    l1_block_hash           VARCHAR NOT NULL
);

-- World Chain Mainnet
CREATE TABLE IF NOT EXISTS world_chain_mainnet (
    id                      SERIAL PRIMARY KEY,
    l2_output_root          VARCHAR NOT NULL,
    l2_output_index         INTEGER NOT NULL,
    l2_block_number         INTEGER NOT NULL,
    l1_timestamp            INTEGER NOT NULL,
    l1_transaction_hash     VARCHAR NOT NULL,
    l1_block_number         INTEGER NOT NULL,
    l1_transaction_index    INTEGER NOT NULL,
    l1_block_hash           VARCHAR NOT NULL
);

-- World Chain Sepolia
CREATE TABLE IF NOT EXISTS world_chain_sepolia (
    id                      SERIAL PRIMARY KEY,
    l2_output_root          VARCHAR NOT NULL,
    l2_output_index         INTEGER NOT NULL,
    l2_block_number         INTEGER NOT NULL,
    l1_timestamp            INTEGER NOT NULL,
    l1_transaction_hash     VARCHAR NOT NULL,
    l1_block_number         INTEGER NOT NULL,
    l1_transaction_index    INTEGER NOT NULL,
    l1_block_hash           VARCHAR NOT NULL
);

-- ============================================================================
-- Arbitrum Tables (Arbitrum, ApeChain)
-- ============================================================================

-- Arbitrum Mainnet
CREATE TABLE IF NOT EXISTS arbitrum_mainnet (
    id                      SERIAL PRIMARY KEY,
    l2_output_root          VARCHAR NOT NULL,
    l2_block_hash           VARCHAR NOT NULL,
    l2_block_number         INTEGER NOT NULL,
    l1_transaction_hash     VARCHAR NOT NULL,
    l1_block_number         INTEGER NOT NULL,
    l1_transaction_index    INTEGER NOT NULL,
    l1_block_hash           VARCHAR NOT NULL
);

-- Arbitrum Sepolia
CREATE TABLE IF NOT EXISTS arbitrum_sepolia (
    id                      SERIAL PRIMARY KEY,
    l2_output_root          VARCHAR NOT NULL,
    l2_block_hash           VARCHAR NOT NULL,
    l2_block_number         INTEGER NOT NULL,
    l1_transaction_hash     VARCHAR NOT NULL,
    l1_block_number         INTEGER NOT NULL,
    l1_transaction_index    INTEGER NOT NULL,
    l1_block_hash           VARCHAR NOT NULL
);

-- Ape Chain Mainnet
CREATE TABLE IF NOT EXISTS ape_chain_mainnet (
    id                      SERIAL PRIMARY KEY,
    l2_output_root          VARCHAR NOT NULL,
    l2_block_hash           VARCHAR NOT NULL,
    l2_block_number         INTEGER NOT NULL,
    l1_transaction_hash     VARCHAR NOT NULL,
    l1_block_number         INTEGER NOT NULL,
    l1_transaction_index    INTEGER NOT NULL,
    l1_block_hash           VARCHAR NOT NULL
);

-- Ape Chain Sepolia
CREATE TABLE IF NOT EXISTS ape_chain_sepolia (
    id                      SERIAL PRIMARY KEY,
    l2_output_root          VARCHAR NOT NULL,
    l2_block_hash           VARCHAR NOT NULL,
    l2_block_number         INTEGER NOT NULL,
    l1_transaction_hash     VARCHAR NOT NULL,
    l1_block_number         INTEGER NOT NULL,
    l1_transaction_index    INTEGER NOT NULL,
    l1_block_hash           VARCHAR NOT NULL
);

-- ============================================================================
-- Fault Dispute Games Tables (Optimism & Base Mainnet & Sepolia)
-- ============================================================================

-- Optimism Mainnet Fault Dispute Games
CREATE TABLE IF NOT EXISTS optimism_mainnet_fault_dispute_games (
    id                      SERIAL PRIMARY KEY,
    game_index              BIGINT NOT NULL,
    game_address            VARCHAR NOT NULL,
    game_type               BIGINT NOT NULL,
    timestamp               BIGINT NOT NULL,
    root_claim              VARCHAR NOT NULL,
    game_state              BIGINT NOT NULL,
    proposer_address        VARCHAR NOT NULL,
    l2_block_number         BIGINT NOT NULL,
    l2_state_root           VARCHAR,
    l2_withdrawal_storage_root VARCHAR,
    l2_block_hash           VARCHAR,
    l1_timestamp            BIGINT NOT NULL,
    l1_transaction_hash     VARCHAR NOT NULL,
    l1_block_number         BIGINT NOT NULL,
    l1_transaction_index    BIGINT NOT NULL,
    l1_block_hash           VARCHAR NOT NULL
);

-- Optimism Sepolia Fault Dispute Games
CREATE TABLE IF NOT EXISTS optimism_sepolia_fault_dispute_games (
    id                      SERIAL PRIMARY KEY,
    game_index              BIGINT NOT NULL,
    game_address            VARCHAR NOT NULL,
    game_type               BIGINT NOT NULL,
    timestamp               BIGINT NOT NULL,
    root_claim              VARCHAR NOT NULL,
    game_state              BIGINT NOT NULL,
    proposer_address        VARCHAR NOT NULL,
    l2_block_number         BIGINT NOT NULL,
    l2_state_root           VARCHAR,
    l2_withdrawal_storage_root VARCHAR,
    l2_block_hash           VARCHAR,
    l1_timestamp            BIGINT NOT NULL,
    l1_transaction_hash     VARCHAR NOT NULL,
    l1_block_number         BIGINT NOT NULL,
    l1_transaction_index    BIGINT NOT NULL,
    l1_block_hash           VARCHAR NOT NULL
);

-- Base Mainnet Fault Dispute Games
CREATE TABLE IF NOT EXISTS base_mainnet_fault_dispute_games (
    id                      SERIAL PRIMARY KEY,
    game_index              BIGINT NOT NULL,
    game_address            VARCHAR NOT NULL,
    game_type               BIGINT NOT NULL,
    timestamp               BIGINT NOT NULL,
    root_claim              VARCHAR NOT NULL,
    game_state              BIGINT NOT NULL,
    proposer_address        VARCHAR NOT NULL,
    l2_block_number         BIGINT NOT NULL,
    l2_state_root           VARCHAR,
    l2_withdrawal_storage_root VARCHAR,
    l2_block_hash           VARCHAR,
    l1_timestamp            BIGINT NOT NULL,
    l1_transaction_hash     VARCHAR NOT NULL,
    l1_block_number         BIGINT NOT NULL,
    l1_transaction_index    BIGINT NOT NULL,
    l1_block_hash           VARCHAR NOT NULL
);

-- Base Sepolia Fault Dispute Games
CREATE TABLE IF NOT EXISTS base_sepolia_fault_dispute_games (
    id                      SERIAL PRIMARY KEY,
    game_index              BIGINT NOT NULL,
    game_address            VARCHAR NOT NULL,
    game_type               BIGINT NOT NULL,
    timestamp               BIGINT NOT NULL,
    root_claim              VARCHAR NOT NULL,
    game_state              BIGINT NOT NULL,
    proposer_address        VARCHAR NOT NULL,
    l2_block_number         BIGINT NOT NULL,
    l2_state_root           VARCHAR,
    l2_withdrawal_storage_root VARCHAR,
    l2_block_hash           VARCHAR,
    l1_timestamp            BIGINT NOT NULL,
    l1_transaction_hash     VARCHAR NOT NULL,
    l1_block_number         BIGINT NOT NULL,
    l1_transaction_index    BIGINT NOT NULL,
    l1_block_hash           VARCHAR NOT NULL
);

-- ============================================================================
-- Indexes for better query performance
-- ============================================================================

-- Indexes on l1_block_number for all OPStack tables
CREATE INDEX IF NOT EXISTS idx_optimism_mainnet_l1_block_number ON optimism_mainnet(l1_block_number);
CREATE INDEX IF NOT EXISTS idx_optimism_sepolia_l1_block_number ON optimism_sepolia(l1_block_number);
CREATE INDEX IF NOT EXISTS idx_base_mainnet_l1_block_number ON base_mainnet(l1_block_number);
CREATE INDEX IF NOT EXISTS idx_base_sepolia_l1_block_number ON base_sepolia(l1_block_number);
CREATE INDEX IF NOT EXISTS idx_zora_mainnet_l1_block_number ON zora_mainnet(l1_block_number);
CREATE INDEX IF NOT EXISTS idx_zora_sepolia_l1_block_number ON zora_sepolia(l1_block_number);
CREATE INDEX IF NOT EXISTS idx_world_chain_mainnet_l1_block_number ON world_chain_mainnet(l1_block_number);
CREATE INDEX IF NOT EXISTS idx_world_chain_sepolia_l1_block_number ON world_chain_sepolia(l1_block_number);

-- Indexes on l1_block_number for all Arbitrum tables
CREATE INDEX IF NOT EXISTS idx_arbitrum_mainnet_l1_block_number ON arbitrum_mainnet(l1_block_number);
CREATE INDEX IF NOT EXISTS idx_arbitrum_sepolia_l1_block_number ON arbitrum_sepolia(l1_block_number);
CREATE INDEX IF NOT EXISTS idx_ape_chain_mainnet_l1_block_number ON ape_chain_mainnet(l1_block_number);
CREATE INDEX IF NOT EXISTS idx_ape_chain_sepolia_l1_block_number ON ape_chain_sepolia(l1_block_number);

-- Indexes on l1_block_number for fault dispute games tables
CREATE INDEX IF NOT EXISTS idx_optimism_mainnet_fdg_l1_block_number ON optimism_mainnet_fault_dispute_games(l1_block_number);
CREATE INDEX IF NOT EXISTS idx_optimism_sepolia_fdg_l1_block_number ON optimism_sepolia_fault_dispute_games(l1_block_number);
CREATE INDEX IF NOT EXISTS idx_base_mainnet_fdg_l1_block_number ON base_mainnet_fault_dispute_games(l1_block_number);
CREATE INDEX IF NOT EXISTS idx_base_sepolia_fdg_l1_block_number ON base_sepolia_fault_dispute_games(l1_block_number);

-- Indexes on l2_block_number for all tables
CREATE INDEX IF NOT EXISTS idx_optimism_mainnet_l2_block_number ON optimism_mainnet(l2_block_number);
CREATE INDEX IF NOT EXISTS idx_optimism_sepolia_l2_block_number ON optimism_sepolia(l2_block_number);
CREATE INDEX IF NOT EXISTS idx_base_mainnet_l2_block_number ON base_mainnet(l2_block_number);
CREATE INDEX IF NOT EXISTS idx_base_sepolia_l2_block_number ON base_sepolia(l2_block_number);
CREATE INDEX IF NOT EXISTS idx_zora_mainnet_l2_block_number ON zora_mainnet(l2_block_number);
CREATE INDEX IF NOT EXISTS idx_zora_sepolia_l2_block_number ON zora_sepolia(l2_block_number);
CREATE INDEX IF NOT EXISTS idx_world_chain_mainnet_l2_block_number ON world_chain_mainnet(l2_block_number);
CREATE INDEX IF NOT EXISTS idx_world_chain_sepolia_l2_block_number ON world_chain_sepolia(l2_block_number);
CREATE INDEX IF NOT EXISTS idx_arbitrum_mainnet_l2_block_number ON arbitrum_mainnet(l2_block_number);
CREATE INDEX IF NOT EXISTS idx_arbitrum_sepolia_l2_block_number ON arbitrum_sepolia(l2_block_number);
CREATE INDEX IF NOT EXISTS idx_ape_chain_mainnet_l2_block_number ON ape_chain_mainnet(l2_block_number);
CREATE INDEX IF NOT EXISTS idx_ape_chain_sepolia_l2_block_number ON ape_chain_sepolia(l2_block_number);
CREATE INDEX IF NOT EXISTS idx_optimism_mainnet_fdg_l2_block_number ON optimism_mainnet_fault_dispute_games(l2_block_number);
CREATE INDEX IF NOT EXISTS idx_optimism_sepolia_fdg_l2_block_number ON optimism_sepolia_fault_dispute_games(l2_block_number);
CREATE INDEX IF NOT EXISTS idx_base_mainnet_fdg_l2_block_number ON base_mainnet_fault_dispute_games(l2_block_number);
CREATE INDEX IF NOT EXISTS idx_base_sepolia_fdg_l2_block_number ON base_sepolia_fault_dispute_games(l2_block_number);

-- Index on game_index for fault dispute games tables
CREATE INDEX IF NOT EXISTS idx_optimism_mainnet_fdg_game_index ON optimism_mainnet_fault_dispute_games(game_index);
CREATE INDEX IF NOT EXISTS idx_optimism_sepolia_fdg_game_index ON optimism_sepolia_fault_dispute_games(game_index);
CREATE INDEX IF NOT EXISTS idx_base_mainnet_fdg_game_index ON base_mainnet_fault_dispute_games(game_index);
CREATE INDEX IF NOT EXISTS idx_base_sepolia_fdg_game_index ON base_sepolia_fault_dispute_games(game_index);

