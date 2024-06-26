#!/bin/bash

# Step 1: Run Docker Compose to build and start your services
echo "Building and starting services using Docker Compose..."
docker-compose up --build -d

# Step 2: Build the Rust application
echo "Building the Rust application..."
cargo build --release

# Step 4: Start the application using PM2
echo "Starting the application using PM2..."
pm2 start pm2.config.js

echo "Setup complete."


# Setup for Postgres without Docker: Check if the database exists and create it if it does not
# DB_EXISTS=$(psql postgresql://postgres:password@localhost:5432/l2indexer -tAc "SELECT 1 FROM pg_database WHERE datname='l2indexer'")
# if [ "$DB_EXISTS" = '1' ]; then
#     echo "Database l2indexer already exists."
# else
#     echo "Database l2indexer does not exist. Creating database..."
#     psql postgresql://postgres:password@localhost:5432/l2indexer -c "CREATE DATABASE l2indexer"
# fi