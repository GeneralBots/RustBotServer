#!/bin/bash
set -e

# Create directories
echo "Creating directories..."
INSTALL_DIR="$HOME/server_binaries"
mkdir -p "$INSTALL_DIR"
mkdir -p "$INSTALL_DIR/config"
mkdir -p "$INSTALL_DIR/data"

# Install system dependencies
echo "Installing system dependencies..."
sudo apt-get install -y \
    apt-transport-https \
    ca-certificates \
    curl \
    software-properties-common \
    gnupg \
    wget \
    unzip \
    tar \
    postgresql-client \
    redis-tools

echo "System dependencies installed"

# Download PostgreSQL binary (using the official package)
# echo "Downloading PostgreSQL..."
# if [ ! -d "$INSTALL_DIR/postgresql" ]; then
#     mkdir -p "$INSTALL_DIR/postgresql"
#     wget -O "$INSTALL_DIR/postgresql/postgresql.tar.gz" "https://get.enterprisedb.com/postgresql/postgresql-14.10-1-linux-x64-binaries.tar.gz"
#     tar -xzf "$INSTALL_DIR/postgresql/postgresql.tar.gz" -C "$INSTALL_DIR/postgresql" --strip-components=1
#     rm "$INSTALL_DIR/postgresql/postgresql.tar.gz"
#     mkdir -p "$INSTALL_DIR/data/postgresql"
# fi

# Download Redis binary
echo "Downloading Redis..."
if [ ! -d "$INSTALL_DIR/redis" ]; then
    mkdir -p "$INSTALL_DIR/redis"
    wget -O "$INSTALL_DIR/redis/redis.tar.gz" "https://download.redis.io/releases/redis-7.2.4.tar.gz"
    tar -xzf "$INSTALL_DIR/redis/redis.tar.gz" -C "$INSTALL_DIR/redis" --strip-components=1
    rm "$INSTALL_DIR/redis/redis.tar.gz"
    mkdir -p "$INSTALL_DIR/data/redis"
fi

# Download Zitadel binary
# echo "Downloading Zitadel..."
# if [ ! -d "$INSTALL_DIR/zitadel" ]; then
#     mkdir -p "$INSTALL_DIR/zitadel"
#     # Get latest release URL
#     ZITADEL_LATEST=$(curl -s https://api.github.com/repos/zitadel/zitadel/releases/latest | grep "browser_download_url.*linux_amd64.tar.gz" | cut -d '"' -f 4)
#     wget -O "$INSTALL_DIR/zitadel/zitadel.tar.gz" "$ZITADEL_LATEST"
#     tar -xzf "$INSTALL_DIR/zitadel/zitadel.tar.gz" -C "$INSTALL_DIR/zitadel"
#     rm "$INSTALL_DIR/zitadel/zitadel.tar.gz"
#     mkdir -p "$INSTALL_DIR/data/zitadel"
    
#     # Create default Zitadel config
#     cat > "$INSTALL_DIR/config/zitadel.yaml" <<EOF
# Log:
#   Level: info
# Database:
#   postgres:
#     Host: localhost
#     Port: 5432
#     Database: zitadel
#     User: postgres
#     Password: postgres
#     SSL:
#       Mode: disable
# EOF
# fi

# Download Stalwart Mail binary
# echo "Downloading Stalwart Mail..."
# if [ ! -d "$INSTALL_DIR/stalwart" ]; then
#     mkdir -p "$INSTALL_DIR/stalwart"
#     # Get latest release URL
#     STALWART_LATEST=$(curl -s https://api.github.com/repos/stalwartlabs/mail-server/releases/latest | grep "browser_download_url.*linux-x86_64.tar.gz" | cut -d '"' -f 4)
#     wget -O "$INSTALL_DIR/stalwart/stalwart.tar.gz" "$STALWART_LATEST"
#     tar -xzf "$INSTALL_DIR/stalwart/stalwart.tar.gz" -C "$INSTALL_DIR/stalwart"
#     rm "$INSTALL_DIR/stalwart/stalwart.tar.gz"
#     mkdir -p "$INSTALL_DIR/data/stalwart"
    
#     # Download config files
#     mkdir -p "$INSTALL_DIR/config/stalwart"
#     wget -O "$INSTALL_DIR/config/stalwart/config.toml" "https://raw.githubusercontent.com/stalwartlabs/mail-server/main/resources/config/config.toml"
# fi

# Download MinIO binary
echo "Downloading MinIO..."
if [ ! -f "$INSTALL_DIR/minio/minio" ]; then
    mkdir -p "$INSTALL_DIR/minio"
    wget -O "$INSTALL_DIR/minio/minio" "https://dl.min.io/server/minio/release/linux-amd64/minio"
    chmod +x "$INSTALL_DIR/minio/minio"
    mkdir -p "$INSTALL_DIR/data/minio"
fi

# Download Redpanda binary
echo "Downloading Redpanda..."
if [ ! -d "$INSTALL_DIR/redpanda" ]; then
    mkdir -p "$INSTALL_DIR/redpanda"
    # Get latest Redpanda binary
    REDPANDA_LATEST=$(curl -s https://api.github.com/repos/redpanda-data/redpanda/releases/latest | grep "browser_download_url.*linux-amd64.zip" | cut -d '"' -f 4)
    wget -O "$INSTALL_DIR/redpanda/redpanda.zip" "$REDPANDA_LATEST"
    unzip -o "$INSTALL_DIR/redpanda/redpanda.zip" -d "$INSTALL_DIR/redpanda"
    rm "$INSTALL_DIR/redpanda/redpanda.zip"
    mkdir -p "$INSTALL_DIR/data/redpanda"
    
    # Create default config
    cat > "$INSTALL_DIR/config/redpanda.yaml" <<EOF
redpanda:
  data_directory: $INSTALL_DIR/data/redpanda
  rpc_server:
    address: 127.0.0.1
    port: 33145
  kafka_api:
    - address: 127.0.0.1
      port: 9092
  admin:
    - address: 127.0.0.1
      port: 9644
EOF
fi

# Download Vector binary
echo "Downloading Vector..."
if [ ! -d "$INSTALL_DIR/vector" ]; then
    mkdir -p "$INSTALL_DIR/vector"
    # Get latest release URL
    VECTOR_LATEST=$(curl -s https://api.github.com/repos/vectordotdev/vector/releases/latest | grep "browser_download_url.*x86_64-unknown-linux-gnu.tar.gz" | head -n 1 | cut -d '"' -f 4)
    wget -O "$INSTALL_DIR/vector/vector.tar.gz" "$VECTOR_LATEST"
    tar -xzf "$INSTALL_DIR/vector/vector.tar.gz" -C "$INSTALL_DIR/vector" --strip-components=1
    rm "$INSTALL_DIR/vector/vector.tar.gz"
    mkdir -p "$INSTALL_DIR/data/vector"
    
    # Create Vector config
    cat > "$INSTALL_DIR/config/vector.toml" <<EOF
[sources.syslog]
type = "syslog"
address = "0.0.0.0:514"
mode = "tcp"

[sources.file_logs]
type = "file"
include = ["$INSTALL_DIR/data/*/logs/*.log"]
ignore_older_secs = 86400 # 1 day

[transforms.parse_logs]
type = "remap"
inputs = ["syslog", "file_logs"]
source = '''
. = parse_syslog!(string!(.message))
'''

[sinks.console]
type = "console"
inputs = ["parse_logs"]
encoding.codec = "json"

[sinks.local_file]
type = "file"
inputs = ["parse_logs"]
path = "$INSTALL_DIR/data/vector/output.log"
encoding.codec = "json"
EOF
fi

echo "All binaries downloaded to $INSTALL_DIR"
echo "Use the start-stop script to manually control all components"