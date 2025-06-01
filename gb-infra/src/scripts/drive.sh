#!/bin/bash

DATA_PATH="/opt/gbo/tenants/$PARAM_TENANT/drive/data"
LOGS_PATH="/opt/gbo/tenants/$PARAM_TENANT/drive/logs"

mkdir -p "${DATA_PATH}" "${LOGS_PATH}"
chmod -R 770 "${DATA_PATH}" "${LOGS_PATH}"
chown -R 100999:100999 "${DATA_PATH}" "${LOGS_PATH}"

lxc launch images:debian/12 "${PARAM_TENANT}-drive" -c security.privileged=true
sleep 15

lxc config device add "${PARAM_TENANT}-drive" storage disk source="${DATA_PATH}" path=/opt/gbo/data
lxc config device add "${PARAM_TENANT}-drive" logs disk source="${LOGS_PATH}" path=/opt/gbo/logs

lxc exec "${PARAM_TENANT}-drive" -- bash -c '

mkdir -p /opt/gbo/logs /opt/gbo/data /opt/gbo/bin
useradd -r -s /bin/false gbuser || true
chown -R gbuser:gbuser /opt/gbo/logs /opt/gbo/data

apt-get update && apt-get install -y wget
wget https://dl.min.io/server/minio/release/linux-amd64/minio -O /opt/gbo/bin/minio
chmod +x /opt/gbo/bin/minio

cat > /etc/systemd/system/drive.service <<EOF
[Unit]
Description=drive
After=network.target

[Service]
Type=simple
User=gbuser
Group=gbuser
Environment="MINIO_ROOT_USER='"${PARAM_DRIVE_USER}"'"
Environment="MINIO_ROOT_PASSWORD='"${PARAM_DRIVE_PASSWORD}"'"
ExecStart=/opt/gbo/bin/minio server --console-address ":'"${PARAM_DRIVE_PORT}"'" /opt/gbo/data
StandardOutput=append:/opt/gbo/logs/output.log
StandardError=append:/opt/gbo/logs/error.log

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable drive
systemctl start drive
'

lxc config device remove "${PARAM_TENANT}-drive" drive-proxy 2>/dev/null || true
lxc config device add "${PARAM_TENANT}-drive" drive-proxy proxy \
    listen=tcp:0.0.0.0:"${PARAM_DRIVE_API_PORT}" \
    connect=tcp:127.0.0.1:"${PARAM_DRIVE_API_PORT}"

lxc config device remove "${PARAM_TENANT}-drive" console-proxy 2>/dev/null || true
lxc config device add "${PARAM_TENANT}-drive" console-proxy proxy \
    listen=tcp:0.0.0.0:"${PARAM_DRIVE_PORT}" \
    connect=tcp:127.0.0.1:"${PARAM_DRIVE_PORT}"