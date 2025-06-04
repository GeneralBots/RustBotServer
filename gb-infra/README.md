
# Backup

## Fastest way to tranfer files between servers over TCP/IP

rsync -avz --progress --bwlimit=0 -e "ssh -p 22 -T -c aes128-gcm@openssh.com -o Compression=no -o IPQoS=throughput" gbbackup@host.com.br:/opt/gbo/backup /home/user/Desktop


# Security
apt update && apt install -y fail2ban 
systemctl enable fail2ban


apt update && apt install -y fail2ban iptables-persistent

systemctl enable fail2ban
systemctl enable netfilter-persistent