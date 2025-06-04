#!/bin/bash

HOST_BASE="/opt/gbo/tenants/$PARAM_TENANT/proxy"
HOST_DATA="$HOST_BASE/data"
HOST_CONF="$HOST_BASE/conf"
HOST_LOGS="$HOST_BASE/logs"

mkdir -p "$HOST_DATA" "$HOST_CONF" "$HOST_LOGS"
chmod -R 750 "$HOST_BASE"

lxc launch images:debian/12 "$PARAM_TENANT"-proxy -c security.privileged=true
sleep 15

lxc exec "$PARAM_TENANT"-proxy -- bash -c "
apt-get update && apt-get install -y curl libcap2-bin
curl -sL \"https://github.com/caddyserver/caddy/releases/download/v2.10.0-beta.3/caddy_2.10.0-beta.3_linux_amd64.tar.gz\" | tar -C /usr/local/bin -xz caddy
chmod 755 /usr/local/bin/caddy
setcap 'cap_net_bind_service=+ep' /usr/local/bin/caddy
useradd --system --no-create-home --shell /usr/sbin/nologin caddy
"

CADDY_UID=$(lxc exec "$PARAM_TENANT"-proxy -- id -u caddy)
CADDY_GID=$(lxc exec "$PARAM_TENANT"-proxy -- id -g caddy)
HOST_CADDY_UID=$((100000 + CADDY_UID))
HOST_CADDY_GID=$((100000 + CADDY_GID))
chown -R "$HOST_CADDY_UID:$HOST_CADDY_GID" "$HOST_BASE"

lxc config device add "$PARAM_TENANT"-proxy proxydata disk source="$HOST_DATA" path=/var/lib/caddy
lxc config device add "$PARAM_TENANT"-proxy proxyconf disk source="$HOST_CONF" path=/etc/caddy
lxc config device add "$PARAM_TENANT"-proxy proxylogs disk source="$HOST_LOGS" path=/var/log/caddy

lxc exec "$PARAM_TENANT"-proxy -- bash -c "
mkdir -p /var/lib/caddy /etc/caddy /var/log/caddy
chown -R caddy:caddy /var/lib/caddy /etc/caddy /var/log/caddy

cat > /etc/caddy/Caddyfile <<EOF
:80 {
    respond \"Welcome to $PARAM_TENANT Proxy\"
    log {
        output file /var/log/caddy/access.log
    }
}
EOF

cat > /etc/systemd/system/caddy.service <<EOF
[Unit]
Description=Caddy
After=network.target

[Service]
User=root
Group=root
ExecStart=/usr/local/bin/caddy run --config /etc/caddy/Caddyfile --adapter caddyfile
ExecReload=/usr/local/bin/caddy reload --config /etc/caddy/Caddyfile --adapter caddyfile
TimeoutStopSec=5s
LimitNOFILE=1048576
LimitNPROC=512
PrivateTmp=true
ProtectSystem=full
AmbientCapabilities=CAP_NET_BIND_SERVICE

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable caddy
systemctl start caddy
"

lxc config device remove "$PARAM_TENANT"-proxy http-proxy 2>/dev/null || true
lxc config device add "$PARAM_TENANT"-proxy http-proxy proxy \
    listen=tcp:0.0.0.0:"$PARAM_HTTP_PORT" \
    connect=tcp:127.0.0.1:"$PARAM_HTTP_PORT"

lxc config device remove "$PARAM_TENANT"-proxy https-proxy 2>/dev/null || true
lxc config device add "$PARAM_TENANT"-proxy https-proxy proxy \
    listen=tcp:0.0.0.0:"$PARAM_HTTPS_PORT" \
    connect=tcp:127.0.0.1:"$PARAM_HTTPS_PORT"