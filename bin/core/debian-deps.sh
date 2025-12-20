#!/bin/bash

## Core deps installer

apt-get update
apt-get install -y git curl ca-certificates iproute2 age

# Sops only available as a binary
SOPS_VERSION="v3.11.0"
ARCH=$(dpkg --print-architecture)
curl -L "https://github.com/getsops/sops/releases/download/${SOPS_VERSION}/sops-${SOPS_VERSION}.linux.${ARCH}" -o /usr/local/bin/sops
chmod +x /usr/local/bin/sops

rm -rf /var/lib/apt/lists/*

# Starship prompt
curl -sS https://starship.rs/install.sh | sh -s -- --yes --bin-dir /usr/local/bin
echo 'export STARSHIP_CONFIG=/starship.toml' >> /root/.bashrc
echo 'eval "$(starship init bash)"' >> /root/.bashrc

