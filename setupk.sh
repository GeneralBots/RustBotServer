#!/bin/bash

# Enable error handling
set -e

# Function to check command status
check_status() {
    if [ $? -eq 0 ]; then
        echo "✅ $1 successful"
    else
        echo "❌ $1 failed"
        exit 1
    fi
}

echo "🚀 Starting Kubernetes installation..."

# Update system
echo "📦 Updating system packages..."
sudo apt-get update && sudo apt-get upgrade -y
check_status "System update"

# Install prerequisites
echo "📦 Installing prerequisites..."
sudo apt-get install -y apt-transport-https ca-certificates curl software-properties-common
check_status "Prerequisites installation"

# Install containerd
echo "🐋 Installing containerd..."
sudo apt-get install -y containerd
check_status "Containerd installation"

# Configure containerd
echo "⚙️ Configuring containerd..."
sudo mkdir -p /etc/containerd
sudo containerd config default | sudo tee /etc/containerd/config.toml > /dev/null
sudo sed -i 's/SystemdCgroup = false/SystemdCgroup = true/' /etc/containerd/config.toml
sudo systemctl restart containerd
sudo systemctl enable containerd
check_status "Containerd configuration"

# Disable swap
echo "⚙️ Disabling swap..."
sudo swapoff -a
sudo sed -i '/swap/d' /etc/fstab
check_status "Swap disabled"

# Load kernel modules
echo "⚙️ Loading kernel modules..."
sudo modprobe overlay
sudo modprobe br_netfilter
check_status "Kernel modules loaded"

# Configure system settings
echo "⚙️ Configuring system settings..."
cat <<EOF | sudo tee /etc/sysctl.d/k8s.conf
net.bridge.bridge-nf-call-iptables  = 1
net.bridge.bridge-nf-call-ip6tables = 1
net.ipv4.ip_forward                 = 1
EOF
sudo sysctl --system
check_status "System settings configuration"

# Add Kubernetes repository
echo "📦 Adding Kubernetes repository..."
sudo mkdir -p /etc/apt/keyrings
curl -fsSL https://pkgs.k8s.io/core:/stable:/v1.28/deb/Release.key | sudo gpg --dearmor -o /etc/apt/keyrings/kubernetes-apt-keyring.gpg
echo 'deb [signed-by=/etc/apt/keyrings/kubernetes-apt-keyring.gpg] https://pkgs.k8s.io/core:/stable:/v1.28/deb/ /' | sudo tee /etc/apt/sources.list.d/kubernetes.list
check_status "Kubernetes repository addition"

# Install Kubernetes components
echo "📦 Installing Kubernetes components..."
sudo apt-get update
sudo apt-get install -y kubelet kubeadm kubectl
sudo apt-mark hold kubelet kubeadm kubectl
check_status "Kubernetes components installation"

# Initialize Kubernetes cluster
echo "🚀 Initializing Kubernetes cluster..."
sudo kubeadm init --pod-network-cidr=10.244.0.0/16
check_status "Kubernetes initialization"

# Configure kubectl
echo "⚙️ Configuring kubectl..."
mkdir -p $HOME/.kube
sudo cp -i /etc/kubernetes/admin.conf $HOME/.kube/config
sudo chown $(id -u):$(id -g) $HOME/.kube/config
check_status "kubectl configuration"

# Install Flannel network plugin
echo "🔌 Installing Flannel network plugin..."
kubectl apply -f https://github.com/flannel-io/flannel/releases/latest/download/kube-flannel.yml
check_status "Flannel installation"

# Allow scheduling on control-plane node
echo "⚙️ Enabling workload scheduling on control-plane..."
kubectl taint nodes --all node-role.kubernetes.io/control-plane- || true
kubectl taint nodes --all node-role.kubernetes.io/master- || true
check_status "Node configuration"

# Verify installation
echo "🔍 Verifying installation..."
kubectl get nodes
check_status "Node verification"

echo "✨ Kubernetes installation completed successfully!"
echo "🔍 Cluster status:"
kubectl cluster-info
echo "📝 Node status:"
kubectl get nodes

# Save cluster join command if needed
echo "💾 Saving cluster join command..."
sudo kubeadm token create --print-join-command > $HOME/k8s_join_command.txt
chmod 600 $HOME/k8s_join_command.txt
echo "Join command saved to $HOME/k8s_join_command.txt"

echo "
✅ Installation complete! 
To start using your cluster:
  kubectl get nodes
  kubectl get pods --all-namespaces

To reset the cluster if needed:
  sudo kubeadm reset
  sudo rm -rf /etc/cni/net.d
  sudo rm -rf $HOME/.kube/config
"