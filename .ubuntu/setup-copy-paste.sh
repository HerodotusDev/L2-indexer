echo "DOCKER" &&\
sudo apt update &&\
sudo apt install docker.io -y &&\
sudo systemctl enable docker &&\
sudo systemctl start docker &&\
sudo usermod -aG docker ${USER} &&\

echo "DOCKER-COMPOSE" &&\
sudo curl -L https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m) -o /usr/local/bin/docker-compose &&\
sudo chmod +x /usr/local/bin/docker-compose &&\

echo "RUST" &&\
sudo apt install build-essential -y &&\
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh &&\
source "$HOME/.cargo/env" &&\
cargo install --force cargo-make &&\

echo "PM2" &&\
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash &&\
source ~/.bashrc &&\
nvm install 22 &&\
npm install pm2 -g &&\
pm2 startup &&\

echo "NGINX" &&\
sudo apt update -y &&\
sudo apt install nginx git make -y &&\
sudo systemctl enable nginx &&\
sudo systemctl start nginx &&\

echo "CERTBOT" &&\
sudo apt install python3-venv python3-pip -y &&\
sudo python3 -m venv /opt/certbot/ &&\
sudo /opt/certbot/bin/pip install --upgrade pip &&\
sudo /opt/certbot/bin/pip install certbot certbot-nginx &&\
sudo ln -s /opt/certbot/bin/certbot /usr/bin/certbot &&\
sudo certbot --version &&\

echo "HTTPS (NGINX + CERTBOT)" &&\
echo "user www-data;
worker_processes auto;

events {
        worker_connections 1024;
}

http {
        server {
                server_name l2-indexer.api.herodotus.cloud;
                location / {
                        proxy_set_header X-Forwarded-For \$remote_addr;
                        proxy_set_header Host \$http_host;
                        proxy_pass \"http://0.0.0.0:8000\";
                }
        }
}" > tmp.conf && sudo cp tmp.conf /etc/nginx/nginx.conf && rm tmp.conf &&\
sudo certbot --nginx -d l2-indexer.api.herodotus.cloud --non-interactive --agree-tos --email aws+api.production@herodotus.dev
