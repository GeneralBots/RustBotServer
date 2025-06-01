for container in $(lxc list -c n --format csv); do
    lxc config set "$container" limits.memory 2048MB
    lxc config set "$container" limits.cpu.allowance "20ms/100ms"
    lxc config set "$container" limits.cpu 1
    lxc config set "$container" limits.cpu.priority 1
done

# Restart all containers (gracefully)
for container in $(lxc list -c n --format csv); do
    echo "Restarting $container..."
    lxc restart "$container"   # --force ensures a hard restart if needed
done

# Check limits for all containers
for container in $(lxc list -c n --format csv); do
    echo "--- $container ---"
    lxc config show "$container" | grep -E "memory|cpu"
done