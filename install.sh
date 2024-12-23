#!/bin/bash
set -e


# Install https transport if not already installed

sudo apt-get install -y apt-transport-https ca-certificates curl software-properties-common gnupg

echo "Repository fixes completed!"

# Install system dependencies
echo "Installing system dependencies..."
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    git \
    clang \
    libclang-dev \
    postgresql \
    postgresql-contrib \
    redis-server \
    libopencv-dev \
    libtesseract-dev \
    cmake \
    protobuf-compiler \
    libprotobuf-dev

# Install Rust if not already installed
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Install kubectl if not present
if ! command -v kubectl &> /dev/null; then
    echo "Installing kubectl..."
    curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
    chmod +x kubectl
    sudo mv kubectl /usr/local/bin/
fi

# Setup project structure
echo "Setting up project structure..."
mkdir -p general-bots
cd general-bots

# Optional: Azure CLI installation
echo "Would you like to install Azure CLI? (y/n)"
read -r response
if [[ "$response" =~ ^([yY][eE][sS]|[yY])+$ ]]
then
    echo "Installing Azure CLI..."
    curl -sL https://packages.microsoft.com/keys/microsoft.asc | gpg --dearmor | sudo tee /etc/apt/trusted.gpg.d/microsoft.gpg > /dev/null
    echo "deb [arch=amd64] https://packages.microsoft.com/repos/azure-cli/ $(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/azure-cli.list
    sudo apt-get update
    sudo apt-get install -y azure-cli
fi

# Optional: HandBrake installation
echo "Would you like to install HandBrake? (y/n)"
read -r response
if [[ "$response" =~ ^([yY][eE][sS]|[yY])+$ ]]
then
    echo "Installing HandBrake..."
    sudo apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 8771ADB0816950D8
    sudo add-apt-repository -y ppa:stebbins/handbrake-releases
    sudo apt-get update
    sudo apt-get install -y handbrake-cli handbrake-gtk
fi

# Build the project
echo "Building the project..."
cargo build

# Run tests
echo "Running tests..."
./run_tests.sh

# Setup database
echo "Setting up PostgreSQL database..."
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Create database and user (with error handling)
    sudo -u postgres psql -c "CREATE DATABASE generalbots;" 2>/dev/null || echo "Database might already exist"
    sudo -u postgres psql -c "CREATE USER gbuser WITH PASSWORD 'gbpassword';" 2>/dev/null || echo "User might already exist"
    sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE generalbots TO gbuser;" 2>/dev/null || echo "Privileges might already be granted"

# Start Redis
echo "Starting Redis service..."
sudo systemctl start redis-server
sudo systemctl enable redis-server

echo "Installation completed!"
echo "Next steps:"
echo "1. Configure your Kubernetes cluster"
echo "2. Update k8s/base/*.yaml files with your configuration"
echo "3. Run ./deploy.sh to deploy to Kubernetes"
echo "4. Check deployment status with: kubectl -n general-bots get pods"

# Print service status
echo -e "\nService Status:"
echo "PostgreSQL status:"
sudo systemctl status postgresql --no-pager
echo -e "\nRedis status:"
sudo systemctl status redis-server --no-pager

sudo apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 8771ADB0816950D8 && sudo apt-get update && sudo apt-get install -y libglib2.0-dev build-essential pkg-config
sudo apt-get install -y libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly gstreamer1.0-libav gstreamer1.0-tools gstreamer1.0-x gstreamer1.0-alsa gstreamer1.0-gl gstreamer1.0-gtk3 gstreamer1.0-qt5 gstreamer1.0-pulseaudio && export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig:$PKG_CONFIG_PATH && echo 'export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig:$PKG_CONFIG_PATH' >> ~/.bashrc && source ~/.bashrc
