#!/bin/bash

# Step 1: Stop all Docker containers managed by docker-compose
echo "Stopping all Docker containers..."
docker-compose down --rmi all --volumes --remove-orphans

# Step 2: Delete specific data directory, including PostgreSQL
echo "Deleting data directory..."
rm -rf ./data

# Step 3: Stop and remove all PM2 processes
echo "Stopping all PM2 processes..."
pm2 delete pm2.config.js

# Step 4: Selete rust build directory
echo "Deleting rust build directory..."
rm -rf ./target

echo "Cleanup complete."


# Cleanup for Postgres without Docker: Remove the l2indexer database
# echo "Removing the 'l2indexer' database..."
# psql postgresql://postgres:password@localhost:5432/postgres -c "DROP DATABASE IF EXISTS l2indexer"