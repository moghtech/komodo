#!/bin/bash

## Core deps installer

apt-get update
apt-get install -y git curl ca-certificates iproute2

rm -rf /var/lib/apt/lists/*

# Starship prompt
curl -sS https://starship.rs/install.sh | sh -s -- --yes --bin-dir /usr/local/bin
echo 'export STARSHIP_CONFIG=/config/starship.toml' >> /root/.bashrc
echo 'eval "$(starship init bash)"' >> /root/.bashrc

