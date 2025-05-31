#!/bin/bash

DIRECTORY_VERSION="v2.71.2"
HOST_BASE="/opt/gbo/tenants/$PARAM_TENANT/directory"
HOST_DATA="$HOST_BASE/data"
HOST_CONF="$HOST_BASE/conf"
HOST_LOGS="$HOST_BASE/logs"

mkdir -p "$HOST_DATA" "$HOST_CONF" "$HOST_LOGS"
chmod -R 750 "$HOST_BASE"

lxc launch images:debian/12 "$PARAM_TENANT"-directory -c security.privileged=true
sleep 15

lxc config device add "$PARAM_TENANT"-directory directorydata disk source="$HOST_DATA" path=/var/lib/zitadel
lxc config device add "$PARAM_TENANT"-directory directoryconf disk source="$HOST_CONF" path=/etc/zitadel
lxc config device add "$PARAM_TENANT"-directory directorylogs disk source="$HOST_LOGS" path=/var/log/zitadel

lxc exec "$PARAM_TENANT"-directory -- bash -c "
apt-get update && apt-get install -y wget
wget -c https://github.com/zitadel/zitadel/releases/download/$DIRECTORY_VERSION/zitadel-linux-amd64.tar.gz -O - | tar -xz -C /usr/local/bin/

useradd -r -s /bin/false zitadel
mkdir -p /var/lib/zitadel /etc/zitadel /var/log/zitadel
chown -R zitadel:zitadel /var/lib/zitadel /etc/zitadel /var/log/zitadel

cat > /etc/systemd/system/directory.service <<EOF
[Unit]
Description=Directory Service
After=network.target

[Service]
Type=simple
User=zitadel
Group=zitadel
Environment=ZITADEL_DEFAULTINSTANCE_INSTANCENAME=$PARAM_TENANT
Environment=ZITADEL_DEFAULTINSTANCE_ORG_NAME=$PARAM_TENANT
Environment=ZITADEL_DATABASE_TABLES_HOST=$PARAM_TABLES_HOST
Environment=ZITADEL_DATABASE_TABLES_PORT=$PARAM_TABLES_PORT
Environment=ZITADEL_DATABASE_TABLES_DATABASE=$PARAM_DIRECTORY_DATABASE
Environment=ZITADEL_DATABASE_TABLES_USER_USERNAME=$PARAM_TABLES_USERNAME
Environment=ZITADEL_DATABASE_TABLES_USER_PASSWORD=$PARAM_TABLES_PASSWORD
Environment=ZITADEL_DATABASE_TABLES_ADMIN_SSL_MODE=disable
Environment=ZITADEL_DATABASE_TABLES_USER_SSL_MODE=disable
Environment=ZITADEL_DATABASE_TABLES_ADMIN_USERNAME=$PARAM_TABLES_USERNAME
Environment=ZITADEL_DATABASE_TABLES_ADMIN_PASSWORD=$PARAM_TABLES_PASSWORD
Environment=ZITADEL_EXTERNALSECURE=true
ExecStart=/usr/local/bin/zitadel start --masterkey $PARAM_DIRECTORY_MASTERKEY --config /etc/zitadel/config.yaml
WorkingDirectory=/var/lib/zitadel
StandardOutput=append:/var/log/zitadel/output.log
StandardError=append:/var/log/zitadel/error.log
Restart=always

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable directory
systemctl start directory
"

lxc config device remove "$PARAM_TENANT"-directory directory-proxy 2>/dev/null || true
lxc config device add "$PARAM_TENANT"-directory directory-proxy proxy \
    listen=tcp:0.0.0.0:"$PARAM_DIRECTORY_PORT" \
    connect=tcp:127.0.0.1:"$PARAM_DIRECTORY_PORT"