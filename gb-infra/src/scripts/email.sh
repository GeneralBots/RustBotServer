#!/bin/bash
PARAM_TENANT="pragmatismo"
PARAM_STALWART_VERSION="latest"
PARAM_OAUTH_PROVIDER="zitadel"
PARAM_OAUTH_CLIENT_ID="SEU_CLIENT_ID"
PARAM_OAUTH_CLIENT_SECRET="SEU_CLIENT_SECRET"
PARAM_OAUTH_AUTH_ENDPOINT="https://login.pragmatismo.com.br/oauth/v2/authorize"
PARAM_OAUTH_TOKEN_ENDPOINT="https://login.pragmatismo.com.br/oauth/v2/token"
PARAM_OAUTH_USERINFO_ENDPOINT="https://login.pragmatismo.com.br/userinfo"
PARAM_OAUTH_SCOPE="openid email profile"
PARAM_STALWART_PORT="8080"

BIN_PATH="/opt/gbo/bin"
CONF_PATH="/opt/gbo/conf.d"
LOGS_PATH="/opt/gbo/tenants/$PARAM_TENANT/stalwart/logs"

mkdir -p "${BIN_PATH}" "${CONF_PATH}" "${LOGS_PATH}"
chmod -R 770 "${BIN_PATH}" "${CONF_PATH}" "${LOGS_PATH}"
chown -R 100999:100999 "${BIN_PATH}" "${CONF_PATH}" "${LOGS_PATH}"

lxc launch images:debian/12 "${PARAM_TENANT}-stalwart" -c security.privileged=true
sleep 15

lxc config device add "${PARAM_TENANT}-stalwart" logs disk source="${LOGS_PATH}" path=/var/log/stalwart

lxc exec "${PARAM_TENANT}-stalwart" -- bash -c '
apt-get update && apt-get install -y wget
wget -c https://github.com/stalwartlabs/mail-server/releases/download/'"${PARAM_STALWART_VERSION}"'/stalwart-mail-x86_64-unknown-linux-gnu.tar.gz -O - | tar -xz -C /usr/local/bin/

useradd -r -s /bin/false stalwart || true
mkdir -p /var/log/stalwart /opt/gbo/bin /opt/gbo/conf.d
chown -R stalwart:stalwart /var/log/stalwart /opt/gbo/bin /opt/gbo/conf.d

cat > /opt/gbo/conf.d/stalwart.toml <<EOF
[oauth]
provider = "'"${PARAM_OAUTH_PROVIDER}"'"
client_id = "'"${PARAM_OAUTH_CLIENT_ID}"'"
client_secret = "'"${PARAM_OAUTH_CLIENT_SECRET}"'"
authorization_endpoint = "'"${PARAM_OAUTH_AUTH_ENDPOINT}"'"
token_endpoint = "'"${PARAM_OAUTH_TOKEN_ENDPOINT}"'"
userinfo_endpoint = "'"${PARAM_OAUTH_USERINFO_ENDPOINT}"'"
scope = "'"${PARAM_OAUTH_SCOPE}"'"
EOF

cat > /etc/systemd/system/stalwart.service <<EOF
[Unit]
Description=Stalwart Mail Server
After=network.target

[Service]
Type=simple
User=stalwart
Group=stalwart
ExecStart=/usr/local/bin/stalwart-mail --config /opt/gbo/conf.d/stalwart.toml
StandardOutput=append:/var/log/stalwart/output.log
StandardError=append:/var/log/stalwart/error.log

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable stalwart
systemctl start stalwart
'

lxc config device remove "${PARAM_TENANT}-stalwart" stalwart-proxy 2>/dev/null || true
lxc config device add "${PARAM_TENANT}-stalwart" stalwart-proxy proxy \
    listen=tcp:0.0.0.0:"${PARAM_STALWART_PORT}" \
    connect=tcp:127.0.0.1:"${PARAM_STALWART_PORT}"