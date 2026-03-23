#!/bin/sh

cargo install typeshare-cli

sudo mkdir -p /etc/komodo/keys
sudo chown -R $(whoami) /etc/komodo

sudo mkdir -p /config/keys
sudo chown -R $(whoami) /config
