#!/bin/bash

## Periphery deps installer

apt-get update
apt-get install -y git curl wget ca-certificates age

install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc
chmod a+r /etc/apt/keyrings/docker.asc

echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian \
  $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
  tee /etc/apt/sources.list.d/docker.list > /dev/null

apt-get update

# apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
apt-get install -y docker-ce-cli docker-buildx-plugin docker-compose-plugin

# Sops only available as a binary
SOPS_VERSION="v3.12.2"
ARCH=$(dpkg --print-architecture)
curl -L "https://github.com/getsops/sops/releases/download/${SOPS_VERSION}/sops-${SOPS_VERSION}.linux.${ARCH}" -o /usr/local/bin/sops
chmod +x /usr/local/bin/sops

rm -rf /var/lib/apt/lists/*

# Starship prompt
curl -sS https://starship.rs/install.sh | sh -s -- --yes --bin-dir /usr/local/bin
echo 'export STARSHIP_CONFIG=/starship.toml' >> /root/.bashrc
echo 'eval "$(starship init bash)"' >> /root/.bashrc

