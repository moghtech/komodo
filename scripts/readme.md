# Periphery setup script

```
usage: setup-periphery [-h] [--version VERSION] [--user] [--root-directory ROOT_DIRECTORY] [--core-address CORE_ADDRESS]
                       [--connect-as CONNECT_AS] [--onboarding-key ONBOARDING_KEY] [--force-service-file FORCE_SERVICE_FILE]
                       [--config-url CONFIG_URL] [--binary-url BINARY_URL]

Install systemd-managed Komodo Periphery

options:
  -h, --help            show this help message and exit
  --version, -v VERSION
                        Install a specific Komodo version, like 'v2.0.0' (default: latest)
  --user, -u            Install systemd '--user' service (default: False)
  --root-directory, -r ROOT_DIRECTORY
                        Specify a specific Periphery root directory. (default: /etc/komodo)
  --core-address, -c CORE_ADDRESS
                        Specify the Komodo Core address for outbound connection. Leave blank to enable inbound connection server. (default: None)
  --connect-as, -n CONNECT_AS
                        Specify the Server name to connect as. Defaults to hostname. (default: hostname)
  --onboarding-key, -k ONBOARDING_KEY
                        Give an onboarding key for automatic Server onboarding into Komodo Core. (default: None)
  --force-service-file FORCE_SERVICE_FILE
                        Recreate the systemd service file even if it already exists. (default: None)
  --config-url CONFIG_URL
                        Use a custom config url. (default:
                        https://raw.githubusercontent.com/moghtech/komodo/refs/heads/main/config/periphery.config.toml)
  --binary-url BINARY_URL
                        Use alternate binary source (default: https://github.com/moghtech/komodo/releases/download)
```

These scripts will set up Komodo Periphery on your hosts, managed by systemd.

*Note*. This script can be **run multiple times without issue**, and it won't change existing config after the first run. Just run it again after a Komodo version release, and it will update the periphery version.

*Note*. The script can usually detect aarch64 system and use the periphery-aarch64 binary.

There's two ways to install periphery: `System` and `User`

## System (requires root)

Note. Run this after switching to root user (eg `sudo su -`).

```sh
curl -sSL https://raw.githubusercontent.com/moghtech/komodo/main/scripts/setup-periphery.py \
  | python3 - --core-address <YOUR-CORE-ADDRESS> \
  --onboarding-key <YOUR-ONBOARDING-KEY>
```

Will install to paths:
- periphery (binary) -> `/usr/local/bin/periphery`
- periphery.service -> `/etc/systemd/system/periphery.service`
- periphery.config.toml -> `/etc/komodo/periphery.config.toml`

## User

*Note*. The user running periphery must be a member of the docker group, in order to use the docker cli without sudo.

```sh
curl -sSL https://raw.githubusercontent.com/moghtech/komodo/main/scripts/setup-periphery.py \
  | python3 - --user --core-address <YOUR-CORE-ADDRESS> \
  --onboarding-key <YOUR-ONBOARDING-KEY>
```

Will install to paths:
- periphery (binary) -> `$HOME/.local/bin`
- periphery.service -> `$HOME/.config/systemd/user/periphery.service`
- periphery.config.toml -> `$HOME/.config/komodo/periphery.config.toml`

*Note*. Ensure the user running periphery has write permissions to the configured `root_directory`.

*Note*. To ensure periphery stays running when your user logs out, use this:
```shell
sudo loginctl enable-linger $USER
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
curl -sSL https://raw.githubusercontent.com/moghtech/komodo/main/scripts/setup-periphery.py \
  | python3 - --core-address <YOUR-CORE-ADDRESS> \
  --onboarding-key <YOUR-ONBOARDING-KEY> \
  --force-service-file
```