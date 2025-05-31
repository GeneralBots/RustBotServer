#!/bin/bash

HOST_BASE="/opt/gbo/tenants/$PARAM_TENANT/meeting"
HOST_DATA="$HOST_BASE/data"
HOST_CONF="$HOST_BASE/conf"
HOST_LOGS="$HOST_BASE/logs"

mkdir -p "$HOST_DATA" "$HOST_CONF" "$HOST_LOGS"
chmod -R 750 "$HOST_BASE"

lxc launch images:debian/12 "$PARAM_TENANT"-meeting -c security.privileged=true
sleep 15

lxc exec "$PARAM_TENANT"-meeting -- bash -c "
apt-get update && apt-get install -y wget coturn
mkdir -p /opt/livekit-server
cd /opt/livekit-server
wget -q https://github.com/livekit/livekit/releases/download/v1.8.4/livekit_1.8.4_linux_amd64.tar.gz
tar -xzf livekit*.tar.gz
chmod +x livekit-server

while netstat -tuln | grep -q \":$PARAM_MEETING_TURN_PORT \"; do
  ((PARAM_MEETING_TURN_PORT++))
done
"

MEETING_UID=$(lxc exec "$PARAM_TENANT"-meeting -- id -u turnserver)
MEETING_GID=$(lxc exec "$PARAM_TENANT"-meeting -- id -g turnserver)
HOST_MEETING_UID=$((100000 + MEETING_UID))
HOST_MEETING_GID=$((100000 + MEETING_GID))
chown -R "$HOST_MEETING_UID:$HOST_MEETING_GID" "$HOST_BASE"

lxc config device add "$PARAM_TENANT"-meeting meetingdata disk source="$HOST_DATA" path=/var/lib/livekit
lxc config device add "$PARAM_TENANT"-meeting meetingconf disk source="$HOST_CONF" path=/etc/livekit
lxc config device add "$PARAM_TENANT"-meeting meetinglogs disk source="$HOST_LOGS" path=/var/log/livekit

lxc exec "$PARAM_TENANT"-meeting -- bash -c "
mkdir -p /var/lib/livekit /etc/livekit /var/log/livekit
chown -R turnserver:turnserver /var/lib/livekit /etc/livekit /var/log/livekit

cat > /etc/systemd/system/livekit.service <<EOF
[Unit]
Description=LiveKit Server
After=network.target

[Service]
User=turnserver
Group=turnserver
WorkingDirectory=/opt/livekit-server
ExecStart=/opt/livekit-server/livekit-server --config /etc/livekit/config.yaml
Restart=always
Environment=TURN_PORT=$PARAM_MEETING_TURN_PORT

[Install]
WantedBy=multi-user.target
EOF

cat > /etc/systemd/system/turnserver.service <<EOF
[Unit]
Description=TURN Server
After=network.target

[Service]
User=turnserver
Group=turnserver
ExecStart=/usr/bin/turnserver -c /etc/livekit/turnserver.conf
Restart=always

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable livekit turnserver
systemctl start livekit turnserver
"

lxc config device remove "$PARAM_TENANT"-meeting meeting-proxy 2>/dev/null || true
lxc config device add "$PARAM_TENANT"-meeting meeting-proxy proxy \
    listen=tcp:0.0.0.0:"$PARAM_MEETING_PORT" \
    connect=tcp:127.0.0.1:"$PARAM_MEETING_PORT"