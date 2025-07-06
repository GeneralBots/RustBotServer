#!/bin/bash
HOST_BASE="/opt/gbo/tenants/$PARAM_TENANT/dns"
HOST_CONF="$HOST_BASE/conf"
HOST_DATA="$HOST_BASE/data"
HOST_LOGS="$HOST_BASE/logs"
mkdir -p "$HOST_BASE" "$HOST_CONF" "$HOST_DATA" "$HOST_LOGS"
chmod -R 750 "$HOST_BASE"

# Clear existing rules
sudo iptables -F

# Allow DNS traffic
sudo iptables -A INPUT -p udp --dport 53 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 53 -j ACCEPT
sudo iptables -A FORWARD -p udp --dport 53 -j ACCEPT
sudo iptables -A FORWARD -p tcp --dport 53 -j ACCEPT

# Enable NAT
sudo iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE

# Save rules (if using iptables-persistent)
sudo netfilter-persistent save

lxc launch images:debian/12 "${PARAM_TENANT}-dns" -c security.privileged=true
until lxc exec "${PARAM_TENANT}-dns" -- true; do sleep 3; done

# Remove existing proxy devices
lxc config device remove "${PARAM_TENANT}-dns" dns-udp
lxc config device remove "${PARAM_TENANT}-dns" dns-tcp

# Add correct proxy configuration
lxc config device add "${PARAM_TENANT}-dns" dns-udp proxy listen=udp:0.0.0.0:53 connect=udp:127.0.0.1:53
lxc config device add "${PARAM_TENANT}-dns" dns-tcp proxy listen=tcp:0.0.0.0:53 connect=tcp:127.0.0.1:53

lxc exec "${PARAM_TENANT}-dns" -- bash -c "
mkdir /opt/gbo
mkdir /opt/gbo/{bin,conf,data,logs} 

echo 'nameserver 8.8.8.8' > /etc/resolv.conf


apt-get upgrade -y && apt-get install -y wget
wget -qO /opt/gbo/bin/coredns https://github.com/coredns/coredns/releases/download/v1.11.1/coredns_1.11.1_linux_amd64.tgz
tar -xzf /opt/gbo/bin/coredns -C /opt/gbo/bin/
useradd --system --no-create-home --shell /bin/false gbuser
setcap cap_net_bind_service=+ep /opt/gbo/bin/coredns


cat > /etc/systemd/system/dns.service <<EOF2
[Unit]
Description=DNS
After=network.target
[Service]
User=gbuser
ExecStart=/opt/gbo/bin/coredns -conf /opt/gbo/conf/Corefile
Restart=always
[Install]
WantedBy=multi-user.target
EOF2

systemctl stop systemd-resolved
systemctl disable systemd-resolved
rm /etc/resolv.conf

systemctl daemon-reload
systemctl enable dns
"

GBUSER_UID=$(lxc exec "${PARAM_TENANT}-dns" -- id -u gbuser)
HOST_UID=$((100000 + GBUSER_UID))
chown -R "$HOST_UID:$HOST_UID" "$HOST_BASE"

lxc exec "${PARAM_TENANT}-dns" -- bash -c "
chown -R gbuser:gbuser /opt/gbo

"

lxc config device add "${PARAM_TENANT}-dns" dnsdata disk source="$HOST_DATA" path=/opt/gbo/data
lxc config device add "${PARAM_TENANT}-dns" dnsconf disk source="$HOST_CONF" path=/opt/gbo/conf
lxc config device add "${PARAM_TENANT}-dns" dnslogs disk source="$HOST_LOGS" path=/opt/gbo/logs

lxc exec "${PARAM_TENANT}-dns" -- systemctl start dns