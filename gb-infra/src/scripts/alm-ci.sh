#!/bin/bash

ALM_CI_NAME="CI"
ALM_CI_LABELS="gbo"

HOST_BASE="/opt/gbo/tenants/$PARAM_TENANT/alm-ci"
HOST_DATA="$HOST_BASE/data"
HOST_CONF="$HOST_BASE/conf"
HOST_LOGS="$HOST_BASE/logs"
BIN_PATH="/opt/gbo/bin"

mkdir -p "$HOST_DATA" "$HOST_CONF" "$HOST_LOGS"
chmod -R 750 "$HOST_BASE"

lxc launch images:debian/12 "${PARAM_TENANT}-alm-ci" -c security.privileged=true
sleep 15

# Add directory mappings before installation
lxc config device add "${PARAM_TENANT}-alm-ci" almdata disk source="$HOST_DATA" path=/opt/gbo/data
lxc config device add "${PARAM_TENANT}-alm-ci" almconf disk source="$HOST_CONF" path=/opt/gbo/conf
lxc config device add "${PARAM_TENANT}-alm-ci" almlogs disk source="$HOST_LOGS" path=/opt/gbo/logs

lxc exec "${PARAM_TENANT}-alm-ci" -- bash -c "
apt-get update && apt-get install -y wget

mkdir -p ${BIN_PATH} /opt/gbo/data /opt/gbo/conf /opt/gbo/logs
wget -O ${BIN_PATH}/forgejo-runner https://code.forgejo.org/forgejo/runner/releases/download/v6.3.1/forgejo-runner-6.3.1-linux-amd64
chmod +x ${BIN_PATH}/forgejo-runner

${BIN_PATH}/forgejo-runner register --no-interactive \
    --name \"${ALM_CI_NAME}\" \
    --instance \"${PARAM_ALM_CI_INSTANCE}\" \
    --token \"${PARAM_ALM_CI_TOKEN}\" \
    --labels \"${ALM_CI_LABELS}\"

cat > /etc/systemd/system/alm-ci.service <<EOF
[Unit]
Description=ALM CI Runner
After=network.target

[Service]
Type=simple
User=root
Group=root
WorkingDirectory=/opt/gbo/data
ExecStart=${BIN_PATH}/forgejo-runner daemon
Restart=always
StandardOutput=append:/opt/gbo/logs/stdout.log
StandardError=append:/opt/gbo/logs/stderr.log

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable alm-ci
systemctl start alm-ci
"

# Fix permissions on host
chown -R 100000:100000 "$HOST_BASE"  # Using default LXC mapping for root