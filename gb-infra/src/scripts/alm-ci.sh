#!/bin/bash

ALM_CI_VERSION="v6.3.1"
ALM_CI_NAME="CI"
ALM_CI_LABELS="pragmatismo.com.br"
ALM_CI_BIN_PATH="/opt/gbo/bin"


mkdir -p "${ALM_CI_BIN_PATH}"
chmod -R 750 "${ALM_CI_BIN_PATH}"
chown -R 100999:100999 "${ALM_CI_BIN_PATH}"

lxc launch images:debian/12 "${PARAM_TENANT}-alm-ci" -c security.privileged=true
sleep 15

lxc exec "${PARAM_TENANT}-alm-ci" -- bash -c "
apt-get update && apt-get install -y wget
wget -O ${ALM_CI_BIN_PATH}/forgejo-runner https://code.forgejo.org/forgejo/runner/releases/download/${ALM_CI_VERSION}/forgejo-runner-${ALM_CI_VERSION}-linux-amd64
chmod +x ${ALM_CI_BIN_PATH}/forgejo-runner

${ALM_CI_BIN_PATH}/forgejo-runner register --no-interactive \
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
ExecStart=${ALM_CI_BIN_PATH}/forgejo-runner daemon
Restart=always

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable alm-ci
systemctl start alm-ci
"