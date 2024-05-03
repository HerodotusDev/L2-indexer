# Step 1: Stop the application using PM2
echo "Starting the application using PM2..."
pm2 stop pm2.config.js

# Step 2: Stop docker container
echo "Stopping the docker container..."
docker compose down

echo "Stopped."