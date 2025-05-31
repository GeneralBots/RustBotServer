#!/bin/bash

ALM_VERSION="v10.0.2"
HOST_BASE="/opt/gbo/tenants/$PARAM_TENANT/alm"
HOST_DATA="$HOST_BASE/data"
HOST_CONF="$HOST_BASE/conf"
HOST_LOGS="$HOST_BASE/logs"

mkdir -p "$HOST_DATA" "$HOST_CONF" "$HOST_LOGS"
chmod -R 750 "$HOST_BASE"

lxc launch images:debian/12 "$PARAM_TENANT"-alm -c security.privileged=true
sleep 15

lxc exec "$PARAM_TENANT"-alm -- bash -c "
apt-get update && apt-get install -y git git-lfs wget
wget https://codeberg.org/forgejo/forgejo/releases/download/$ALM_VERSION/forgejo-$ALM_VERSION-linux-amd64 -O /usr/local/bin/forgejo
chmod +x /usr/local/bin/forgejo
useradd --system --no-create-home --shell /bin/false forgejo
"

FORGEJO_UID=$(lxc exec "$PARAM_TENANT"-alm -- id -u forgejo)
FORGEJO_GID=$(lxc exec "$PARAM_TENANT"-alm -- id -g forgejo)
HOST_FORGEJO_UID=$((100000 + FORGEJO_UID))
HOST_FORGEJO_GID=$((100000 + FORGEJO_GID))
chown -R "$HOST_FORGEJO_UID:$HOST_FORGEJO_GID" "$HOST_BASE"

lxc config device add "$PARAM_TENANT"-alm almdata disk source="$HOST_DATA" path=/var/lib/forgejo
lxc config device add "$PARAM_TENANT"-alm almconf disk source="$HOST_CONF" path=/etc/forgejo
lxc config device add "$PARAM_TENANT"-alm almlogs disk source="$HOST_LOGS" path=/var/log/forgejo

lxc exec "$PARAM_TENANT"-alm -- bash -c "
mkdir -p /var/lib/forgejo /etc/forgejo /var/log/forgejo
chown -R forgejo:forgejo /var/lib/forgejo /etc/forgejo /var/log/forgejo

cat > /etc/systemd/system/forgejo.service <<EOF
[Unit]
Description=Forgejo
After=network.target

[Service]
User=forgejo
Group=forgejo
WorkingDirectory=/var/lib/forgejo
ExecStart=/usr/local/bin/forgejo web --config /etc/forgejo/app.ini
Restart=always
Environment=USER=forgejo HOME=/var/lib/forgejo

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable forgejo
systemctl start forgejo
"

lxc config device remove "$PARAM_TENANT"-alm alm-proxy 2>/dev/null || true
lxc config device add "$PARAM_TENANT"-alm alm-proxy proxy \
    listen=tcp:0.0.0.0:"$PARAM_ALM_PORT" \
    connect=tcp:127.0.0.1:"$PARAM_ALM_PORT"