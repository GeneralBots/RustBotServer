#!/bin/bash

PUBLIC_INTERFACE="eth0"                 # Your host's public network interface

# Enable IP forwarding
echo "[HOST] Enabling IP forwarding..."
echo "net.ipv4.ip_forward=1" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p

# Configure firewall
echo "[HOST] Configuring firewall..."
sudo iptables -A FORWARD -i $PUBLIC_INTERFACE -o lxcbr0 -p tcp -m multiport --dports 25,80,110,143,465,587,993,995,4190 -j ACCEPT
sudo iptables -A FORWARD -i lxcbr0 -o $PUBLIC_INTERFACE -m state --state RELATED,ESTABLISHED -j ACCEPT
sudo iptables -t nat -A POSTROUTING -o $PUBLIC_INTERFACE -j MASQUERADE

# Save iptables rules permanently (adjust based on your distro)
if command -v iptables-persistent >/dev/null; then
    sudo iptables-save | sudo tee /etc/iptables/rules.v4
fi

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

# Install Stalwart Mail
echo "[CONTAINER] Installing Stalwart Mail..."
lxc exec "$PARAM_TENANT"-email -- bash -c "
apt-get update && apt-get install -y wget
wget -O /tmp/stalwart.tar.gz https://github.com/stalwartlabs/stalwart/releases/download/v0.12.3/stalwart-x86_64-unknown-linux-gnu.tar.gz
tar -xzf /tmp/stalwart.tar.gz -C /tmp
mkdir -p /opt/gbo/bin
mv /tmp/stalwart /opt/gbo/bin/stalwart-mail
chmod +x /opt/gbo/bin/stalwart-mail
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
cat > /etc/systemd/system/email.service <<EOF
[Unit]
Description=Email Service
After=network.target

[Service]
Type=simple
User=email
Group=email
ExecStart=/opt/gbo/bin/stalwart-mail --config /opt/gbo/conf/config.toml
WorkingDirectory=/opt/gbo/data
StandardOutput=append:/opt/gbo/logs/output.log
StandardError=append:/opt/gbo/logs/error.log
Restart=always

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable email
systemctl start email
"

# ------------------------- PORT FORWARDING -------------------------

# Get container IP
CONTAINER_IP=$(lxc list "$PARAM_TENANT"-email -c 4 --format csv | awk '{print $1}')

# Setup port forwarding
echo "[HOST] Setting up port forwarding..."
declare -A PORTS=(
  ["email"]="$PARAM_EMAIL_SMTP_PORT"
  ["http"]="$PARAM_EMAIL_HTTP_PORT" 
  ["imap"]="$PARAM_EMAIL_IMAP_PORT"
  ["imaps"]="$PARAM_EMAIL_IMAPS_PORT"
  ["pop3"]="$PARAM_EMAIL_POP3_PORT"
  ["pop3s"]="$PARAM_EMAIL_POP3S_PORT"
  ["submission"]="$PARAM_EMAIL_SUBMISSION_PORT"
  ["submissions"]="$PARAM_EMAIL_SUBMISSIONS_PORT"
  ["sieve"]="$PARAM_EMAIL_SIEVE_PORT"
)

for service in "${!PORTS[@]}"; do
    # Container proxy device
    lxc config device remove "$PARAM_TENANT"-email "$service-proxy" 2>/dev/null || true
    lxc config device add "$PARAM_TENANT"-email "$service-proxy" proxy \
      listen=tcp:0.0.0.0:"${PORTS[$service]}" \
      connect=tcp:127.0.0.1:"${PORTS[$service]}"
    
    # Host port forwarding
    sudo iptables -t nat -A PREROUTING -i $PUBLIC_INTERFACE -p tcp --dport "${PORTS[$service]}" -j DNAT --to-destination "$CONTAINER_IP":"${PORTS[$service]}"
done

lxc exec $PARAM_TENANT-email -- sudo setcap 'cap_net_bind_service=+ep' /opt/gbo/bin/stalwart-mail

# Save iptables rules again
if command -v iptables-persistent >/dev/null; then
    sudo iptables-save | sudo tee /etc/iptables/rules.v4
fi
