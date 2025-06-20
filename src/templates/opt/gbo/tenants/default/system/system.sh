#!/bin/bash
STORAGE_PATH="/opt/gbo/tenants/$PARAM_TENANT/system/data"
LOGS_PATH="/opt/gbo/tenants/$PARAM_TENANT/system/logs"

mkdir -p "${STORAGE_PATH}" "${LOGS_PATH}"
chmod -R 770 "${STORAGE_PATH}" "${LOGS_PATH}"
chown -R 100999:100999 "${STORAGE_PATH}" "${LOGS_PATH}"

lxc launch images:debian/12 "${PARAM_TENANT}-system" -c security.privileged=true
sleep 15

lxc config device add "${PARAM_TENANT}-system" storage disk source="${STORAGE_PATH}" path=/data
lxc config device add "${PARAM_TENANT}-system" logs disk source="${LOGS_PATH}" path=/var/log/minio

lxc exec "${PARAM_TENANT}-system" -- bash -c '

apt-get update && apt-get install -y wget
wget https://dl.min.io/server/minio/release/linux-amd64/minio -O /usr/local/bin/minio
chmod +x /usr/local/bin/minio

useradd -r -s /bin/false minio-user || true
mkdir -p /var/log/minio /data
chown -R minio-user:minio-user /var/log/minio /data

cat > /etc/systemd/system/minio.service <<EOF
[Unit]
Description=MinIO
After=network.target

[Service]
Type=simple
User=minio-user
Group=minio-user
Environment="MINIO_ROOT_USER='"${PARAM_system_USER}"'"
Environment="MINIO_ROOT_PASSWORD='"${PARAM_system_PASSWORD}"'"
ExecStart=/usr/local/bin/minio server --console-address ":'"${PARAM_system_PORT}"'" /data
StandardOutput=append:/var/log/minio/output.log
StandardError=append:/var/log/minio/error.log

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable minio
systemctl start minio
'

lxc config device remove "${PARAM_TENANT}-system" minio-proxy 2>/dev/null || true
lxc config device add "${PARAM_TENANT}-system" minio-proxy proxy \
    listen=tcp:0.0.0.0:"${PARAM_system_API_PORT}" \
    connect=tcp:127.0.0.1:"${PARAM_system_API_PORT}"

lxc config device remove "${PARAM_TENANT}-system" console-proxy 2>/dev/null || true
lxc config device add "${PARAM_TENANT}-system" console-proxy proxy \
    listen=tcp:0.0.0.0:"${PARAM_system_PORT}" \
    connect=tcp:127.0.0.1:"${PARAM_system_PORT}"