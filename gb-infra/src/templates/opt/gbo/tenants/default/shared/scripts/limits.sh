#!/usr/bin/env bash

# Define container limits in an associative array
declare -A container_limits=(
    # Pattern       Memory    CPU Allowance
    ["*alm*"]="5126MB:15ms/100ms"
    ["*email*"]="1024MB:15ms/100ms"
    ["*webmail*"]="1024MB:20ms/100ms"
    ["*bot*"]="2048MB:20ms/100ms"
    ["*drive*"]="1024MB:20ms/100ms"
)

# Default values (for containers that don't match any pattern)
DEFAULT_MEMORY="1024MB"
DEFAULT_CPU_ALLOWANCE="15ms/100ms"
CPU_COUNT=1
CPU_PRIORITY=10

# Configure all containers
for container in $(lxc list -c n --format csv); do
    echo "Configuring $container..."
    
    memory=$DEFAULT_MEMORY
    cpu_allowance=$DEFAULT_CPU_ALLOWANCE
    
    # Check if container matches any pattern
    for pattern in "${!container_limits[@]}"; do
        if [[ $container == $pattern ]]; then
            IFS=':' read -r memory cpu_allowance <<< "${container_limits[$pattern]}"
            break
        fi
    done
    
    # Apply configuration
    lxc config set "$container" limits.memory "$memory"
    lxc config set "$container" limits.cpu.allowance "$cpu_allowance"
    lxc config set "$container" limits.cpu "$CPU_COUNT"
    lxc config set "$container" limits.cpu.priority "$CPU_PRIORITY"
done

# Restart all containers
echo "Restarting containers..."
for container in $(lxc list -c n --format csv); do
    echo "Restarting $container..."
    lxc restart "$container"
done

# Verify configuration
echo "Verifying limits..."
for container in $(lxc list -c n --format csv); do
    echo "--- $container ---"
    lxc config show "$container" | grep -E "memory|cpu"
done