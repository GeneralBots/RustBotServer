#!/bin/bash
PARAM_RC_VERSION="1.6.6"

HOST_BASE="/opt/$PARAM_WEBMAIL_DOMAIN"
HOST_DATA="$HOST_BASE/data"
HOST_CONF="$HOST_BASE/conf"
HOST_LOGS="$HOST_BASE/logs"

mkdir -p "$HOST_DATA" "$HOST_CONF" "$HOST_LOGS"
chmod -R 750 "$HOST_BASE"

lxc launch images:debian/12 "$PARAM_TENANT"-webmail -c security.privileged=true
sleep 15

lxc exec "$PARAM_TENANT"-webmail -- bash -c "
apt-get update && apt-get install -y software-properties-common wget
add-apt-repository ppa:ondrej/php -y
apt-get update && apt-get install -y \
php8.1 php8.1-fpm php8.1-imap php8.1-pgsql php8.1-mbstring \
php8.1-xml php8.1-curl php8.1-zip php8.1-cli php8.1-intl \
php8.1-dom composer npm roundcube-plugins roundcube-plugins-extra roundcube-pgsql

npm install -g less less-plugin-clean-css

mkdir -p $HOST_BASE
wget -q https://github.com/roundcube/roundcubemail/releases/download/$PARAM_RC_VERSION/roundcubemail-$PARAM_RC_VERSION-complete.tar.gz
tar -xzf roundcubemail-*.tar.gz
mv roundcubemail-$PARAM_RC_VERSION/* $HOST_BASE
rm -rf roundcubemail-*

chown -R www-data:www-data $HOST_BASE
chmod 750 $HOST_BASE
find $HOST_BASE -type d -exec chmod 750 {} \;
find $HOST_BASE -type f -exec chmod 640 {} \;
mkdir $HOST_LOGS
"

WEBMAIL_UID=$(lxc exec "$PARAM_TENANT"-webmail -- id -u www-data)
WEBMAIL_GID=$(lxc exec "$PARAM_TENANT"-webmail -- id -g www-data)
HOST_WEBMAIL_UID=$((100000 + WEBMAIL_UID))
HOST_WEBMAIL_GID=$((100000 + WEBMAIL_GID))
chown -R "$HOST_WEBMAIL_UID:$HOST_WEBMAIL_GID" "$HOST_BASE"

lxc config device add "$PARAM_TENANT"-webmail webmaildata disk source="$HOST_DATA" path=/var/lib/roundcube
lxc config device add "$PARAM_TENANT"-webmail webmailconf disk source="$HOST_CONF" path=/etc/roundcube
lxc config device add "$PARAM_TENANT"-webmail webmaillogs disk source="$HOST_LOGS" path=/var/log/roundcube

lxc exec "$PARAM_TENANT"-webmail -- bash -c "
cat > /etc/systemd/system/webmail.service <<EOF
[Unit]
Description=Roundcube Webmail
After=network.target

[Service]
User=www-data
Group=www-data
WorkingDirectory=$HOST_BASE
ExecStart=/usr/bin/php -S 0.0.0.0:$PARAM_WEBMAIL_PORT -t $HOST_BASE/public_html
Restart=always

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable webmail
systemctl start webmail
systemctl restart php8.1-fpm
"

lxc config device remove "$PARAM_TENANT"-webmail webmail-proxy 2>/dev/null || true
lxc config device add "$PARAM_TENANT"-webmail webmail-proxy proxy \
    listen=tcp:0.0.0.0:"$PARAM_WEBMAIL_PORT" \
    connect=tcp:127.0.0.1:"$PARAM_WEBMAIL_PORT"