printf "%-20s %-10s %-10s %-10s %-6s %s\n" "CONTAINER" "USED" "AVAIL" "TOTAL" "USE%" "MOUNT"
for container in $(lxc list -c n --format csv); do
    disk_info=$(lxc exec $container -- df -h / --output=used,avail,size,pcent | tail -n 1)
    printf "%-20s %s\n" "$container" "$disk_info"
done