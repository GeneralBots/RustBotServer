#!/bin/bash

HOST_BASE="/opt/gbo/tenants/$PARAM_TENANT/botserver"
HOST_DATA="$HOST_BASE/data"
HOST_CONF="$HOST_BASE/conf"
HOST_LOGS="$HOST_BASE/logs"

mkdir -p "$HOST_DATA" "$HOST_CONF" "$HOST_LOGS"
chmod -R 750 "$HOST_BASE"

lxc launch images:debian/12 "$PARAM_TENANT"-botserver -c security.privileged=true
sleep 15

lxc exec "$PARAM_TENANT"-botserver -- bash -c "
apt-get update && apt-get install -y \
build-essential cmake git pkg-config libjpeg-dev libtiff-dev \
libpng-dev libavcodec-dev libavformat-dev libswscale-dev \
libv4l-dev libatlas-base-dev gfortran python3-dev cpulimit \
expect libxtst-dev libpng-dev

export OPENCV4NODEJS_DISABLE_AUTOBUILD=1
export OPENCV_LIB_DIR=/usr/lib/x86_64-linux-gnu

useradd --system --no-create-home --shell /bin/false botserver
"

BOT_UID=$(lxc exec "$PARAM_TENANT"-botserver -- id -u botserver)
BOT_GID=$(lxc exec "$PARAM_TENANT"-botserver -- id -g botserver)
HOST_BOT_UID=$((100000 + BOT_UID))
HOST_BOT_GID=$((100000 + BOT_GID))
chown -R "$HOST_BOT_UID:$HOST_BOT_GID" "$HOST_BASE"

lxc config device add "$PARAM_TENANT"-botserver botdata disk source="$HOST_DATA" path=/var/lib/botserver
lxc config device add "$PARAM_TENANT"-botserver botconf disk source="$HOST_CONF" path=/etc/botserver
lxc config device add "$PARAM_TENANT"-botserver botlogs disk source="$HOST_LOGS" path=/var/log/botserver

lxc exec "$PARAM_TENANT"-botserver -- bash -c "
mkdir -p /var/lib/botserver /etc/botserver /var/log/botserver
chown -R botserver:botserver /var/lib/botserver /etc/botserver /var/log/botserver

cat > /etc/systemd/system/botserver.service <<EOF
[Unit]
Description=Bot Server
After=network.target

[Service]
User=botserver
Group=botserver
WorkingDirectory=/var/lib/botserver
ExecStart=/usr/bin/node /var/lib/botserver/main.js
Restart=always
Environment=PORT=$PARAM_BOT_PORT

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable botserver
systemctl start botserver
"

lxc config device remove "$PARAM_TENANT"-botserver bot-proxy 2>/dev/null || true
lxc config device add "$PARAM_TENANT"-botserver bot-proxy proxy \
    listen=tcp:0.0.0.0:"$PARAM_BOT_PORT" \
    connect=tcp:127.0.0.1:"$PARAM_BOT_PORT"