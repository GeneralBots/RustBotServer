#!/bin/bash

# Configuration
ALM_CI_NAME="CI"
ALM_CI_LABELS="gbo"
FORGEJO_RUNNER_VERSION="v6.3.1"
FORGEJO_RUNNER_BINARY="forgejo-runner-6.3.1-linux-amd64"
CONTAINER_IMAGE="images:debian/12"
 
# Paths
HOST_BASE="/opt/gbo/tenants/$PARAM_TENANT/alm-ci"
HOST_DATA="$HOST_BASE/data"
HOST_CONF="$HOST_BASE/conf"
HOST_LOGS="$HOST_BASE/logs"
BIN_PATH="/opt/gbo/bin"
CONTAINER_NAME="${PARAM_TENANT}-alm-ci"

# Create host directories
mkdir -p "$HOST_DATA" "$HOST_CONF" "$HOST_LOGS" || exit 1
chmod -R 750 "$HOST_BASE" || exit 1

# Launch container
if ! lxc launch "$CONTAINER_IMAGE" "$CONTAINER_NAME"; then
    echo "Failed to launch container"
    exit 1
fi

# Wait for container to be ready
for i in {1..10}; do
    if lxc exec "$CONTAINER_NAME" -- bash -c "true"; then
        break
    fi
    sleep 3
done


# Container setup
lxc exec "$CONTAINER_NAME" -- bash -c "
set -e

# Update and install dependencies
apt-get update && apt-get install -y wget || { echo 'Package installation failed'; exit 1; }

# Create directories
mkdir -p \"$BIN_PATH\" /opt/gbo/data /opt/gbo/conf /opt/gbo/logs || { echo 'Directory creation failed'; exit 1; }

# Download and install forgejo-runner
wget -O \"$BIN_PATH/forgejo-runner\" \"https://code.forgejo.org/forgejo/runner/releases/download/$FORGEJO_RUNNER_VERSION/$FORGEJO_RUNNER_BINARY\" || { echo 'Download failed'; exit 1; }
chmod +x \"$BIN_PATH/forgejo-runner\" || { echo 'chmod failed'; exit 1; }

cd \"$BIN_PATH\"

# Register runner
\"$BIN_PATH/forgejo-runner\" register --no-interactive \\
    --name \"$ALM_CI_NAME\" \\
    --instance \"$PARAM_ALM_CI_INSTANCE\" \\
    --token \"$PARAM_ALM_CI_TOKEN\" \\
    --labels \"$ALM_CI_LABELS\" || { echo 'Runner registration failed'; exit 1; }

chown -R gbuser:gbuser /opt/gbo/data /opt/gbo/conf /opt/gbo/logs /opt/gbo/bin
"

# Set permissions
echo "[CONTAINER] Setting permissions..."
EMAIL_UID=$(lxc exec "$PARAM_TENANT"-alm-ci -- id -u gbuser)
EMAIL_GID=$(lxc exec "$PARAM_TENANT"-alm-ci -- id -g gbuser)
HOST_EMAIL_UID=$((100000 + EMAIL_UID))
HOST_EMAIL_GID=$((100000 + EMAIL_GID))
sudo chown -R "$HOST_EMAIL_UID:$HOST_EMAIL_GID" "$HOST_BASE"


# Add directory mappings
lxc config device add "$CONTAINER_NAME" almdata disk source="$HOST_DATA" path=/opt/gbo/data || exit 1
lxc config device add "$CONTAINER_NAME" almconf disk source="$HOST_CONF" path=/opt/gbo/conf || exit 1
lxc config device add "$CONTAINER_NAME" almlogs disk source="$HOST_LOGS" path=/opt/gbo/logs || exit 1

lxc exec "$CONTAINER_NAME" -- bash -c "
# Create systemd service
cat > /etc/systemd/system/alm-ci.service <<EOF
[Unit]
Description=ALM CI Runner
After=network.target

[Service]
Type=simple
User=root
Group=root
WorkingDirectory=$BIN_PATH
ExecStart=$BIN_PATH/forgejo-runner daemon
Restart=always

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
systemctl daemon-reload || { echo 'daemon-reload failed'; exit 1; }
systemctl enable alm-ci || { echo 'enable service failed'; exit 1; }
systemctl start alm-ci || { echo 'start service failed'; exit 1; }
"
