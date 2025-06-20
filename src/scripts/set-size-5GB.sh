for container in $(lxc list --format csv -c n); do
    echo "Processing $container..."
    
    # Stop container safely
    lxc stop "$container" 

    # Set new 5GB limit (works for most drivers)
    if ! lxc config device override "$container" root size=5GB; then
        echo "Failed to set config, trying alternative method..."
        lxc config device set "$container" root size=5GB
    fi
    
    # Start container
    lxc start "$container"
    
    # Find root device inside container
    ROOT_DEV=$(lxc exec "$container" -- df / --output=source | tail -1)
    
    # Resize filesystem (with proper error handling)
    if lxc exec "$container" -- which resize2fs >/dev/null 2>&1; then
        echo "Resizing filesystem for $container..."
        if [[ "$ROOT_DEV" == /dev/* ]]; then
            lxc exec "$container" -- growpart "$(dirname "$ROOT_DEV")" "$(basename "$ROOT_DEV")"
            lxc exec "$container" -- resize2fs "$ROOT_DEV"
        else
            echo "Non-standard root device $ROOT_DEV - manual resize needed"
        fi
    else
        echo "resize2fs not available in $container - install it first"
    fi
    
    echo "Completed $container"
done