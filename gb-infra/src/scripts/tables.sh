
HOST_BASE="/opt/gbo/tenants/$PARAM_TENANT/tables"
HOST_DATA="$HOST_BASE/data"
HOST_CONF="$HOST_BASE/conf"
HOST_LOGS="$HOST_BASE/logs"

mkdir -p "$HOST_DATA" "$HOST_CONF" "$HOST_LOGS"

lxc launch images:debian/12 "$PARAM_TENANT"-tables -c security.privileged=true

until lxc exec "$PARAM_TENANT"-tables -- test -f /bin/bash; do

    sleep 5
done
sleep 10

lxc exec "$PARAM_TENANT"-tables -- bash -c "
set -e
export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y wget gnupg2 sudo lsb-release
CODENAME=\$(lsb_release -cs)

wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | gpg --dearmor > /etc/apt/trusted.gpg.d/postgresql.gpg
apt-get install -y postgresql-14 postgresql-client-14
if ! id postgres &>/dev/null; then

    exit 1
fi
systemctl stop postgresql@14-main 2>/dev/null || systemctl stop postgresql 2>/dev/null || true
"

POSTGRES_UID=$(lxc exec "$PARAM_TENANT"-tables -- id -u postgres)
POSTGRES_GID=$(lxc exec "$PARAM_TENANT"-tables -- id -g postgres)

HOST_POSTGRES_UID=$((100000 + POSTGRES_UID))
HOST_POSTGRES_GID=$((100000 + POSTGRES_GID))

chown -R "$HOST_POSTGRES_UID:$HOST_POSTGRES_GID" "$HOST_BASE"
chmod -R 750 "$HOST_BASE"

lxc config device add "$PARAM_TENANT"-tables pgdata disk source="$HOST_DATA" path=/var/lib/postgresql/14/main
lxc config device add "$PARAM_TENANT"-tables pgconf disk source="$HOST_CONF" path=/etc/postgresql/14/main
lxc config device add "$PARAM_TENANT"-tables pglogs disk source="$HOST_LOGS" path=/var/log/postgresql

mkdir -p /var/lib/postgresql/14/main
mkdir -p /etc/postgresql/14/main
mkdir -p /var/log/postgresql
chown -R postgres:postgres /var/lib/postgresql/14/main
chown -R postgres:postgres /etc/postgresql/14/main
chown -R postgres:postgres /var/log/postgresql
chmod 700 /var/lib/postgresql/14/main

sudo -u postgres /usr/lib/postgresql/14/bin/initdb -D /var/lib/postgresql/14/main

cat > /etc/postgresql/14/main/postgresql.conf <<EOF
data_directory = '/var/lib/postgresql/14/main'
hba_file = '/etc/postgresql/14/main/pg_hba.conf'
ident_file = '/etc/postgresql/14/main/pg_ident.conf'
listen_addresses = '*'
port = $PARAM_TABLES_PORT
max_connections = 100
shared_buffers = 128MB
log_destination = 'stderr'
logging_collector = on
log_directory = '/var/log/postgresql'
log_filename = 'postgresql-%Y-%m-%d_%H%M%S.log'
EOF

cat > /etc/postgresql/14/main/pg_hba.conf <<EOF
local   all             postgres                                peer
local   all             all                                     peer
host    all             all             127.0.0.1/32            md5
host    all             all             ::1/128                 md5
host    all             all             0.0.0.0/0               md5
systemctl start postgresql@14-main
systemctl enable postgresql@14-main
EOF

lxc config device remove "$PARAM_TENANT"-tables postgres-proxy 2>/dev/null || true
lxc config device add "$PARAM_TENANT"-tables postgres-proxy proxy \
    listen=tcp:0.0.0.0:"$PARAM_TABLES_PORT" \
    connect=tcp:127.0.0.1:"$PARAM_TABLES_PORT"

cd /var/lib/postgresql
until sudo -u postgres psql -p $PARAM_TABLES_PORT -c '\q' 2>/dev/null; do 

sleep 3
sudo -u "$PARAM_TABLES_USER" psql -p $PARAM_TABLES_PORT -c \"CREATE USER $PARAM_TENANT WITH PASSWORD '$PARAM_TABLES_PASSWORD';\" 2>/dev/null
sudo -u "$PARAM_TABLES_USER" psql -p $PARAM_TABLES_PORT -c \"CREATE DATABASE ${PARAM_TENANT}_db OWNER $PARAM_TENANT;\" 2>/dev/null
sudo -u "$PARAM_TABLES_USER" psql -p $PARAM_TABLES_PORT -c \"GRANT ALL PRIVILEGES ON DATABASE ${PARAM_TENANT}_db TO $PARAM_TENANT;\" 2>/dev/null
