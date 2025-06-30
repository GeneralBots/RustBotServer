#!/bin/bash
HOST_BASE="/opt/gbo/tenants/$PARAM_TENANT/system"
HOST_DATA="$HOST_BASE/data"
HOST_CONF="$HOST_BASE/conf"
HOST_LOGS="$HOST_BASE/logs"
HOST_BIN="$HOST_BASE/bin"
BIN_PATH="/opt/gbo/bin"
CONTAINER_NAME="${PARAM_TENANT}-system"

# Create host directories
mkdir -p "$HOST_DATA" "$HOST_CONF" "$HOST_LOGS" || exit 1
chmod -R 750 "$HOST_BASE" || exit 1


lxc launch images:debian/12 $CONTAINER_NAME -c security.privileged=true
sleep 15

lxc exec $CONTAINER_NAME -- bash -c '

apt-get update && apt-get install -y wget

useradd -r -s /bin/false gbuser || true
mkdir -p /opt/gbo/logs /opt/gbo/bin /opt/gbo/data /opt/gbo/conf
chown -R gbuser:gbuser /opt/gbo/

cat > /etc/systemd/system/system.service <<EOF
[Unit]
Description=General Bots System Service
After=network.target

[Service]
Type=simple
User=gbuser
Group=gbuser
ExecStart=/opt/gbo/bin/gbserver
StandardOutput=append:/opt/gbo/logs/output.log
StandardError=append:/opt/gbo/logs/error.log

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable system
systemctl start system
'

lxc config device add $CONTAINER_NAME bin disk source="${HOST_BIN}" path=/opt/gbo/bin
lxc config device add $CONTAINER_NAME data disk source="${HOST_DATA}" path=/opt/gbo/data
lxc config device add $CONTAINER_NAME conf disk source="${HOST_CONF}" path=/opt/gbo/conf
lxc config device add $CONTAINER_NAME logs disk source="${HOST_LOGS}" path=/opt/gbo/logs


lxc config device remove $CONTAINER_NAME proxy 2>/dev/null || true
lxc config device add $CONTAINER_NAME proxy proxy \
    listen=tcp:0.0.0.0:"${PARAM_SYSTEM_PORT}" \
    connect=tcp:127.0.0.1:"${PARAM_SYSTEM_PORT}"
