#!/bin/bash

# Configuration
PARAM_TENANT=""
PARAM_PORT="4444"
PARAM_PASSWORD=""

# Host paths
HOST_BASE="/opt/gbo/tenants/$PARAM_TENANT/tables"
HOST_DATA="$HOST_BASE/data"
HOST_CONF="$HOST_BASE/conf"
HOST_LOGS="$HOST_BASE/logs"

# Create fresh directories with proper permissions
echo "Creating host directories..."
mkdir -p "$HOST_DATA" "$HOST_CONF" "$HOST_LOGS"

# Launch container first to get the postgres UID
echo "Launching container..."
lxc launch images:debian/12 "$PARAM_TENANT"-tables -c security.privileged=true

# Wait for container to be ready
echo "Waiting for container to start..."
until lxc exec "$PARAM_TENANT"-tables -- test -f /bin/bash; do
    echo "Container not ready, waiting..."
    sleep 5
done
sleep 10

# Install PostgreSQL 14
echo "Installing PostgreSQL 14..."
lxc exec "$PARAM_TENANT"-tables -- bash -c "
set -e
export DEBIAN_FRONTEND=noninteractive

# Update package list and install prerequisites
apt-get update
apt-get install -y wget gnupg2 sudo lsb-release

# Add PostgreSQL repository with proper variable expansion
CODENAME=\$(lsb_release -cs)
echo \"deb http://apt.postgresql.org/pub/repos/apt \${CODENAME}-pgdg main\" > /etc/apt/sources.list.d/pgdg.list

# Add repository key
wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | gpg --dearmor > /etc/apt/trusted.gpg.d/postgresql.gpg

# Update package list with new repository
apt-get update

# Install PostgreSQL 14 specifically
apt-get install -y postgresql-14 postgresql-client-14

# Verify installation
if ! id postgres &>/dev/null; then
    echo 'ERROR: PostgreSQL installation failed - postgres user not created'
    exit 1
fi

# Stop PostgreSQL service
systemctl stop postgresql@14-main 2>/dev/null || systemctl stop postgresql 2>/dev/null || true
"

# Get the postgres UID/GID from inside the container
echo "Getting postgres user information..."
POSTGRES_UID=$(lxc exec "$PARAM_TENANT"-tables -- id -u postgres)
POSTGRES_GID=$(lxc exec "$PARAM_TENANT"-tables -- id -g postgres)

echo "Container postgres UID: $POSTGRES_UID, GID: $POSTGRES_GID"

# Set correct ownership on host directories
# LXD maps container UID 999 to host UID 100999, container UID 70 to host UID 100070, etc.
HOST_POSTGRES_UID=$((100000 + POSTGRES_UID))
HOST_POSTGRES_GID=$((100000 + POSTGRES_GID))

echo "Setting host directory ownership to UID: $HOST_POSTGRES_UID, GID: $HOST_POSTGRES_GID"
chown -R "$HOST_POSTGRES_UID:$HOST_POSTGRES_GID" "$HOST_BASE"
chmod -R 750 "$HOST_BASE"

# Now mount directories after setting permissions
echo "Mounting directories..."
lxc config device add "$PARAM_TENANT"-tables pgdata disk source="$HOST_DATA" path=/var/lib/postgresql/14/main
lxc config device add "$PARAM_TENANT"-tables pgconf disk source="$HOST_CONF" path=/etc/postgresql/14/main
lxc config device add "$PARAM_TENANT"-tables pglogs disk source="$HOST_LOGS" path=/var/log/postgresql

# Initialize and configure PostgreSQL
echo "Configuring PostgreSQL..."
lxc exec "$PARAM_TENANT"-tables -- bash -c "
set -e

# Ensure directories exist and have correct permissions inside container
mkdir -p /var/lib/postgresql/14/main
mkdir -p /etc/postgresql/14/main
mkdir -p /var/log/postgresql
chown -R postgres:postgres /var/lib/postgresql/14/main
chown -R postgres:postgres /etc/postgresql/14/main
chown -R postgres:postgres /var/log/postgresql
chmod 700 /var/lib/postgresql/14/main

# Initialize database in the mounted directory
sudo -u postgres /usr/lib/postgresql/14/bin/initdb -D /var/lib/postgresql/14/main

# Create PostgreSQL configuration
cat > /etc/postgresql/14/main/postgresql.conf <<EOF
data_directory = '/var/lib/postgresql/14/main'
hba_file = '/etc/postgresql/14/main/pg_hba.conf'
ident_file = '/etc/postgresql/14/main/pg_ident.conf'
listen_addresses = '*'
port = $PARAM_PORT
max_connections = 100
shared_buffers = 128MB
log_destination = 'stderr'
logging_collector = on
log_directory = '/var/log/postgresql'
log_filename = 'postgresql-%Y-%m-%d_%H%M%S.log'
EOF

# Configure authentication
cat > /etc/postgresql/14/main/pg_hba.conf <<EOF
# PostgreSQL Client Authentication Configuration File
local   all             postgres                                peer
local   all             all                                     peer
host    all             all             127.0.0.1/32            md5
host    all             all             ::1/128                 md5
host    all             all             0.0.0.0/0               md5
EOF

# Set proper ownership again after configuration
chown -R postgres:postgres /var/lib/postgresql/14/main
chown -R postgres:postgres /etc/postgresql/14/main
chown -R postgres:postgres /var/log/postgresql

# Start PostgreSQL service
systemctl start postgresql@14-main
systemctl enable postgresql@14-main

# Wait for PostgreSQL to be ready
sleep 10
"

# Set up port forwarding
echo "Setting up port forwarding..."
lxc config device remove "$PARAM_TENANT"-tables postgres-proxy 2>/dev/null || true
lxc config device add "$PARAM_TENANT"-tables postgres-proxy proxy \
    listen=tcp:0.0.0.0:"$PARAM_PORT" \
    connect=tcp:127.0.0.1:"$PARAM_PORT"

# Create database user and database
echo "Creating database user and database..."
lxc exec "$PARAM_TENANT"-tables -- bash -c "
set -e
cd /var/lib/postgresql

# Wait for PostgreSQL to be fully ready on the correct port
until sudo -u postgres psql -p $PARAM_PORT -c '\q' 2>/dev/null; do 
    echo 'Waiting for PostgreSQL to be ready on port $PARAM_PORT ...'
    sleep 3
done

# Create user and database (suppress directory warnings)
sudo -u postgres psql -p $PARAM_PORT -c \"CREATE USER $PARAM_TENANT WITH PASSWORD '$PARAM_PASSWORD';\" 2>/dev/null
sudo -u postgres psql -p $PARAM_PORT -c \"CREATE DATABASE ${PARAM_TENANT}_db OWNER $PARAM_TENANT;\" 2>/dev/null
sudo -u postgres psql -p $PARAM_PORT -c \"GRANT ALL PRIVILEGES ON DATABASE ${PARAM_TENANT}_db TO $PARAM_TENANT;\" 2>/dev/null

echo 'PostgreSQL setup completed successfully!'
"

echo "Container setup complete!"
echo "Connection details:"
echo "  Host: localhost"
echo "  Port: $PARAM_PORT"
echo "  Database: ${PARAM_TENANT}_db"
echo "  Username: $PARAM_TENANT"
echo "  Password: $PARAM_PASSWORD"

# Test connection on the correct port
echo "Testing connection..."
lxc exec "$PARAM_TENANT"-tables -- bash -c "cd /var/lib/postgresql && sudo -u postgres psql -p $PARAM_PORT -c '\l'" 2>/dev/null | grep "${PARAM_TENANT}_db" && echo "✓ Database created successfully" || echo "✗ Database creation failed"