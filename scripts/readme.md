# Periphery setup script

These scripts will set up Komodo Periphery on your hosts, managed by systemd.

*Note*. This script can be **run multiple times without issue**, and it won't change existing config after the first run. Just run it again after a Komodo version release, and it will update the periphery version.

*Note*. The script can usually detect aarch64 system and use the periphery-aarch64 binary.

There's two ways to install periphery: `System` and `User`

## System (requires root)

Note. Run this after switching to root user (eg `sudo su -`).

```sh
curl -sSL https://raw.githubusercontent.com/moghtech/komodo/main/scripts/setup-periphery.py | python3
```

Will install to paths:
- periphery (binary) -> `/usr/local/bin/periphery`
- periphery.service -> `/etc/systemd/system/periphery.service`
- periphery.config.toml -> `/etc/komodo/periphery.config.toml`

## User

*Note*. The user running periphery must be a member of the docker group, in order to use the docker cli without sudo.

```sh
curl -sSL https://raw.githubusercontent.com/moghtech/komodo/main/scripts/setup-periphery.py | python3 - --user
```

Will install to paths:
- periphery (binary) -> `$HOME/.local/bin`
- periphery.service -> `$HOME/.config/systemd/user/periphery.service`
- periphery.config.toml -> `$HOME/.config/komodo/periphery.config.toml`

*Note*. Ensure the user running periphery has write permissions to the configured folders `repo_dir`, `stack_dir`, and `ssl_key_file` / `ssl_cert_file` parent folder.
This allows periphery to clone repos, write compose files, and generate ssl certs.

*Note*. To ensure periphery stays running when your user logs out, use `sudo loginctl enable-linger $USER`.

For example in `periphery.config.toml`, running under `ubuntu` user:
```toml
repo_dir = "/home/ubuntu/.komodo/repos"
stack_dir = "/home/ubuntu/.komodo/stacks"

ssl_enabled = true
ssl_key_file = "/home/ubuntu/.komodo/ssl/key.pem"
ssl_cert_file = "/home/ubuntu/.komodo/ssl/cert.pem"
```

For additional information on configuring the systemd service, see the systemd service file documentation here:
[https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html](https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html).

## Force Service File Recreation

Usually the installer will only create the systemd service files (`periphery.service`) if one doesn't already exist.
This means the user is free to customize it to fit their needs, such as changing the `User=` running the binary.

You can change this behavior by passing `--force-service-file`, which will restore the service file
to the current default.

Example:

```sh
curl -sSL https://raw.githubusercontent.com/moghtech/komodo/main/scripts/setup-periphery.py | python3 - --force-service-file
```