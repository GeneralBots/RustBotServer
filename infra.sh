#!/bin/bash

# Base directory
INSTALL_DIR="$HOME/server_binaries"
PID_DIR="$INSTALL_DIR/pids"
LOG_DIR="$INSTALL_DIR/logs"

# Create directories if they don't exist
mkdir -p "$PID_DIR"
mkdir -p "$LOG_DIR"

# Function to start all services
start_all() {
    echo "Starting all services..."
    
    # Start PostgreSQL
    if [ ! -f "$PID_DIR/postgres.pid" ]; then
        echo "Starting PostgreSQL..."
        if [ ! -d "$INSTALL_DIR/data/postgresql/base" ]; then
            echo "Initializing PostgreSQL database..."
            "$INSTALL_DIR/postgresql/bin/initdb" -D "$INSTALL_DIR/data/postgresql"
        fi
        
        "$INSTALL_DIR/postgresql/bin/pg_ctl" -D "$INSTALL_DIR/data/postgresql" -l "$LOG_DIR/postgresql.log" start
        echo $! > "$PID_DIR/postgres.pid"
        
        # Create database and user after a short delay
        sleep 5
        "$INSTALL_DIR/postgresql/bin/createdb" -h localhost generalbots || echo "Database might already exist"
        "$INSTALL_DIR/postgresql/bin/createuser" -h localhost gbuser || echo "User might already exist"
        "$INSTALL_DIR/postgresql/bin/psql" -h localhost -c "ALTER USER gbuser WITH PASSWORD 'gbpassword';" || echo "Password might already be set"
        "$INSTALL_DIR/postgresql/bin/psql" -h localhost -c "GRANT ALL PRIVILEGES ON DATABASE generalbots TO gbuser;" || echo "Privileges might already be granted"
        
        # Create database for Zitadel
        "$INSTALL_DIR/postgresql/bin/createdb" -h localhost zitadel || echo "Zitadel database might already exist"
    else
        echo "PostgreSQL already running"
    fi
    
    # Start Redis
    if [ ! -f "$PID_DIR/redis.pid" ]; then
        echo "Starting Redis..."
        "$INSTALL_DIR/redis/src/redis-server" --daemonize yes --dir "$INSTALL_DIR/data/redis" --logfile "$LOG_DIR/redis.log"
        echo $(pgrep -f "redis-server") > "$PID_DIR/redis.pid"
    else
        echo "Redis already running"
    fi
    
    # Start Zitadel
    if [ ! -f "$PID_DIR/zitadel.pid" ]; then
        echo "Starting Zitadel..."
        "$INSTALL_DIR/zitadel/zitadel" start --config "$INSTALL_DIR/config/zitadel.yaml" > "$LOG_DIR/zitadel.log" 2>&1 &
        echo $! > "$PID_DIR/zitadel.pid"
    else
        echo "Zitadel already running"
    fi
    
    # Start Stalwart Mail
    if [ ! -f "$PID_DIR/stalwart.pid" ]; then
        echo "Starting Stalwart Mail Server..."
        "$INSTALL_DIR/stalwart/stalwart-mail" --config "$INSTALL_DIR/config/stalwart/config.toml" > "$LOG_DIR/stalwart.log" 2>&1 &
        echo $! > "$PID_DIR/stalwart.pid"
    else
        echo "Stalwart Mail already running"
    fi
    
    # Start MinIO
    if [ ! -f "$PID_DIR/minio.pid" ]; then
        echo "Starting MinIO..."
        MINIO_ROOT_USER=minioadmin MINIO_ROOT_PASSWORD=minioadmin "$INSTALL_DIR/minio/minio" server "$INSTALL_DIR/data/minio" --console-address :9001 > "$LOG_DIR/minio.log" 2>&1 &
        echo $! > "$PID_DIR/minio.pid"
    else
        echo "MinIO already running"
    fi
    
    # Start Redpanda
    if [ ! -f "$PID_DIR/redpanda.pid" ]; then
        echo "Starting Redpanda..."
        "$INSTALL_DIR/redpanda/bin/redpanda" --config "$INSTALL_DIR/config/redpanda.yaml" start > "$LOG_DIR/redpanda.log" 2>&1 &
        echo $! > "$PID_DIR/redpanda.pid"
    else
        echo "Redpanda already running"
    fi
    
    # Start Vector
    if [ ! -f "$PID_DIR/vector.pid" ]; then
        echo "Starting Vector..."
        "$INSTALL_DIR/vector/bin/vector" --config "$INSTALL_DIR/config/vector.toml" > "$LOG_DIR/vector.log" 2>&1 &
        echo $! > "$PID_DIR/vector.pid"
    else
        echo "Vector already running"
    fi
    
    echo "All services started"
    echo "To check status: ./$(basename $0) status"
}

# Function to stop all services
stop_all() {
    echo "Stopping all services..."
    
    # Stop Vector
    if [ -f "$PID_DIR/vector.pid" ]; then
        echo "Stopping Vector..."
        kill -TERM $(cat "$PID_DIR/vector.pid") 2>/dev/null || echo "Vector was not running"
        rm "$PID_DIR/vector.pid" 2>/dev/null
    fi
    
    # Stop Redpanda
    if [ -f "$PID_DIR/redpanda.pid" ]; then
        echo "Stopping Redpanda..."
        kill -TERM $(cat "$PID_DIR/redpanda.pid") 2>/dev/null || echo "Redpanda was not running"
        rm "$PID_DIR/redpanda.pid" 2>/dev/null
    fi
    
    # Stop MinIO
    if [ -f "$PID_DIR/minio.pid" ]; then
        echo "Stopping MinIO..."
        kill -TERM $(cat "$PID_DIR/minio.pid") 2>/dev/null || echo "MinIO was not running"
        rm "$PID_DIR/minio.pid" 2>/dev/null
    fi
    
    # Stop Stalwart Mail
    if [ -f "$PID_DIR/stalwart.pid" ]; then
        echo "Stopping Stalwart Mail Server..."
        kill -TERM $(cat "$PID_DIR/stalwart.pid") 2>/dev/null || echo "Stalwart Mail was not running"
        rm "$PID_DIR/stalwart.pid" 2>/dev/null
    fi
    
    # Stop Zitadel
    if [ -f "$PID_DIR/zitadel.pid" ]; then
        echo "Stopping Zitadel..."
        kill -TERM $(cat "$PID_DIR/zitadel.pid") 2>/dev/null || echo "Zitadel was not running"
        rm "$PID_DIR/zitadel.pid" 2>/dev/null
    fi
    
    # Stop Redis
    if [ -f "$PID_DIR/redis.pid" ]; then
        echo "Stopping Redis..."
        "$INSTALL_DIR/redis/src/redis-cli" shutdown 2>/dev/null || echo "Redis CLI not available"
        kill -TERM $(cat "$PID_DIR/redis.pid") 2>/dev/null || echo "Redis was not running"
        rm "$PID_DIR/redis.pid" 2>/dev/null
    fi
    
    # Stop PostgreSQL
    if [ -f "$PID_DIR/postgres.pid" ]; then
        echo "Stopping PostgreSQL..."
        "$INSTALL_DIR/postgresql/bin/pg_ctl" -D "$INSTALL_DIR/data/postgresql" stop 2>/dev/null || echo "PostgreSQL was not running"
        rm "$PID_DIR/postgres.pid" 2>/dev/null
    fi
    
    echo "All services stopped"
}

# Function to check status of all services
check_status() {
    echo "Checking status of all services..."
    
    # Check PostgreSQL
    if [ -f "$PID_DIR/postgres.pid" ] && ps -p $(cat "$PID_DIR/postgres.pid") > /dev/null 2>&1; then
        echo "PostgreSQL: Running (PID: $(cat "$PID_DIR/postgres.pid"))"
    else
        if pgrep -f "postgres" > /dev/null; then
            echo "PostgreSQL: Running (PID: $(pgrep -f "postgres" | head -1))"
        else
            echo "PostgreSQL: Not running"
        fi
    fi
    
    # Check Redis
    if [ -f "$PID_DIR/redis.pid" ] && ps -p $(cat "$PID_DIR/redis.pid") > /dev/null 2>&1; then
        echo "Redis: Running (PID: $(cat "$PID_DIR/redis.pid"))"
    else
        if pgrep -f "redis-server" > /dev/null; then
            echo "Redis: Running (PID: $(pgrep -f "redis-server" | head -1))"
        else
            echo "Redis: Not running"
        fi
    fi
    
    # Check Zitadel
    if [ -f "$PID_DIR/zitadel.pid" ] && ps -p $(cat "$PID_DIR/zitadel.pid") > /dev/null 2>&1; then
        echo "Zitadel: Running (PID: $(cat "$PID_DIR/zitadel.pid"))"
    else
        if pgrep -f "zitadel" > /dev/null; then
            echo "Zitadel: Running (PID: $(pgrep -f "zitadel" | head -1))"
        else
            echo "Zitadel: Not running"
        fi
    fi
    
    # Check Stalwart Mail
    if [ -f "$PID_DIR/stalwart.pid" ] && ps -p $(cat "$PID_DIR/stalwart.pid") > /dev/null 2>&1; then
        echo "Stalwart Mail: Running (PID: $(cat "$PID_DIR/stalwart.pid"))"
    else
        if pgrep -f "stalwart-mail" > /dev/null; then
            echo "Stalwart Mail: Running (PID: $(pgrep -f "stalwart-mail" | head -1))"
        else
            echo "Stalwart Mail: Not running"
        fi
    fi
    
    # Check MinIO
    if [ -f "$PID_DIR/minio.pid" ] && ps -p $(cat "$PID_DIR/minio.pid") > /dev/null 2>&1; then
        echo "MinIO: Running (PID: $(cat "$PID_DIR/minio.pid"))"
    else
        if pgrep -f "minio" > /dev/null; then
            echo "MinIO: Running (PID: $(pgrep -f "minio" | head -1))"
        else
            echo "MinIO: Not running"
        fi
    fi
    
    # Check Redpanda
    if [ -f "$PID_DIR/redpanda.pid" ] && ps -p $(cat "$PID_DIR/redpanda.pid") > /dev/null 2>&1; then
        echo "Redpanda: Running (PID: $(cat "$PID_DIR/redpanda.pid"))"
    else
        if pgrep -f "redpanda" > /dev/null; then
            echo "Redpanda: Running (PID: $(pgrep -f "redpanda" | head -1))"
        else
            echo "Redpanda: Not running"
        fi
    fi
    
    # Check Vector
    if [ -f "$PID_DIR/vector.pid" ] && ps -p $(cat "$PID_DIR/vector.pid") > /dev/null 2>&1; then
        echo "Vector: Running (PID: $(cat "$PID_DIR/vector.pid"))"
    else
        if pgrep -f "vector" > /dev/null; then
            echo "Vector: Running (PID: $(pgrep -f "vector" | head -1))"
        else
            echo "Vector: Not running"
        fi
    fi
}

# Function to restart all services
restart_all() {
    echo "Restarting all services..."
    stop_all
    sleep 3
    start_all
}

# Function to show logs
show_logs() {
    echo "Available logs:"
    ls -la "$LOG_DIR"
    echo ""
    echo "Use 'tail -f $LOG_DIR/[logfile]' to view a specific log"
}

# Check command-line arguments
case "$1" in
    start)
        start_all
        ;;
    stop)
        stop_all
        ;;
    restart)
        restart_all
        ;;
    status)
        check_status
        ;;
    logs)
        show_logs
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status|logs}"
        exit 1
        ;;
esac

exit 0