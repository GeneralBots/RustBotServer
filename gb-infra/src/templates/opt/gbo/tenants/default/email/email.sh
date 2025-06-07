#!/bin/bash


# ------------------------- CONTAINER SETUP -------------------------

# Create directory structure
echo "[CONTAINER] Creating directories..."
HOST_BASE="/opt/gbo/tenants/$PARAM_TENANT/email"
HOST_DATA="$HOST_BASE/data"
HOST_CONF="$HOST_BASE/conf"
HOST_LOGS="$HOST_BASE/logs"

sudo mkdir -p "$HOST_DATA" "$HOST_CONF" "$HOST_LOGS"
sudo chmod -R 750 "$HOST_BASE"

# Launch container
echo "[CONTAINER] Launching LXC container..."
lxc launch images:debian/12 "$PARAM_TENANT"-email -c security.privileged=true
sleep 15

echo "[CONTAINER] Installing Stalwart Mail..."
lxc exec "$PARAM_TENANT"-email -- bash -c "
apt-get update && apt-get install -y wget libcap2-bin
wget -O /tmp/stalwart.tar.gz https://github.com/stalwartlabs/stalwart/releases/download/v0.12.4/stalwart-x86_64-unknown-linux-gnu.tar.gz
                             
tar -xzf /tmp/stalwart.tar.gz -C /tmp
mkdir -p /opt/gbo/bin
mv /tmp/stalwart /opt/gbo/bin/stalwart
chmod +x /opt/gbo/bin/stalwart
rm /tmp/stalwart.tar.gz
useradd --system --no-create-home --shell /bin/false email
mkdir -p /opt/gbo/data /opt/gbo/conf /opt/gbo/logs
chown -R email:email /opt/gbo/data /opt/gbo/conf /opt/gbo/logs /opt/gbo/bin
"

# Set permissions
echo "[CONTAINER] Setting permissions..."
EMAIL_UID=$(lxc exec "$PARAM_TENANT"-email -- id -u email)
EMAIL_GID=$(lxc exec "$PARAM_TENANT"-email -- id -g email)
HOST_EMAIL_UID=$((100000 + EMAIL_UID))
HOST_EMAIL_GID=$((100000 + EMAIL_GID))
sudo chown -R "$HOST_EMAIL_UID:$HOST_EMAIL_GID" "$HOST_BASE"

# Mount directories
echo "[CONTAINER] Mounting directories..."
lxc config device add "$PARAM_TENANT"-email emaildata disk source="$HOST_DATA" path=/opt/gbo/data
lxc config device add "$PARAM_TENANT"-email emailconf disk source="$HOST_CONF" path=/opt/gbo/conf
lxc config device add "$PARAM_TENANT"-email emaillogs disk source="$HOST_LOGS" path=/opt/gbo/logs

# Create systemd service
echo "[CONTAINER] Creating email service..."
lxc exec "$PARAM_TENANT"-email -- bash -c "
chown -R email:email /opt/gbo/data /opt/gbo/conf /opt/gbo/logs /opt/gbo/bin

cat > /etc/systemd/system/email.service <<EOF
[Unit]
Description=Email Service
After=network.target

[Service]
Type=simple
User=email
Group=email
ExecStart=/opt/gbo/bin/stalwart-mail --config /opt/gbo/conf/config.toml
WorkingDirectory=/opt/gbo/bin
Restart=always

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable email
systemctl start email
"

