#!/bin/bash

# Step 1: Stop all Docker containers managed by docker-compose
echo "Stopping all Docker containers..."
docker-compose down

# Step 2: Remove all Docker containers (optional, if not using docker-compose)
# docker rm $(docker ps -a -q) -f

# Step 3: Remove all Docker volumes
echo "Removing all Docker volumes..."
docker volume rm $(docker volume ls -q)

# Step 4: Delete specific data directory for PostgreSQL (adjust the path as needed)
echo "Deleting PostgreSQL data directory..."
rm -rf ./data/postgres-data

# Step 5: Remove the l2indexer database
echo "Removing the 'l2indexer' database..."
psql postgresql://postgres:password@localhost:5432/postgres -c "DROP DATABASE IF EXISTS l2indexer"

# Step 6: Stop and remove all PM2 processes
echo "Stopping all PM2 processes..."
pm2 kill

echo "Cleanup complete."
