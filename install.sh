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
    cmake \
    protobuf-compiler \
    libprotobuf-dev
sudo apt reinstall libssl-dev
sudo apt install -y pkg-config libssl-dev libleptonica-dev
sudo apt install -y libglib2.0-dev libleptonica-dev pkg-config
sudo apt install -y build-essential clang libclang-dev libc-dev
sudo apt install -y libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev


# Install Rust if not already installed
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

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


# Print service status
echo -e "\nService Status:"
echo "PostgreSQL status:"
sudo systemctl status postgresql --no-pager
echo -e "\nRedis status:"
sudo systemctl status redis-server --no-pager