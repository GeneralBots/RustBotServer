export container="pragmatismo-alm-ci"
lxc stop "$container"

lxc config device override "$container" root size=15GB
lxc config device set "$container" root size=15GB
lxc start "$container"
ROOT_DEV=$(lxc exec "$container" -- df / --output=source | tail -1)

lxc exec "$container" -- growpart "$(dirname "$ROOT_DEV")" "$(basename "$ROOT_DEV")"
lxc exec "$container" -- resize2fs "$ROOT_DEV"
