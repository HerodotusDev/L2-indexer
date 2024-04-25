# Step 1: Build the Rust application
echo "Building the Rust application..."
cargo build --release

# Step 2: Start the application using PM2
echo "Starting the application using PM2..."
pm2 start pm2.config.js

echo "Restart complete."