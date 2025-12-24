# Version Upgrades

Most version upgrades only require a redeployment of the Core container after pulling the latest version, and are backward compatible with the periphery clients, which may be updated later on as convenient. This is the default, and will be the case unless specifically mentioned in the [version release notes](https://github.com/moghtech/komodo/releases).

Some Core API upgrades may change behavior such as building / cloning, and require updating the Periphery binaries to match the Core version before this functionality can be restored. This will be specifically mentioned in the release notes.

## Updating to Komodo v2

Komodo v2 introduces a new connection and authentication method between Komodo Core and the Periphery agents running on your Servers, as well as breaking changes to the `komodo.execute_terminal` and `komodo.execute_container_exec` Action methods.

Apart from the `komodo.execute_terminal` Action methods, It is fully backward compatible with Komodo v1 configuration.

### Update Core and Periphery to v2

The first step is to update both Core and Periphery to v2 versions along with some configuration changes.

:::note
Starting with v2, Komodo will not publish images with the `latest` tag in favor of Semver (`2`, `2.0`, `2.0.0`).
This prevents unintented major version upgrades when using auto updaters.
:::

In the Komodo Core compose service, update the image to `:2` tag, and add a mount to `/config/keys`.
```yaml
services:
  core:
    image: ghcr.io/moghtech/komodo-core:2
    init: true # This is a recommended addition regardless of version
    volumes:
      - keys:/config/keys
      - (...unchanged)
    (...unchanged)

volumes:
  keys:
  (...unchanged)
```

If you are running Komodo Periphery in a container, you also need the `:2` tag for the image, and the keys will default to being stored in `$PERIPHERY_ROOT_DIRECTORY/keys` which should already be mounted.
Systemd Periphery users just need to update their Periphery binary version.


After getting both Core and Periphery running, everything should already work correctly at this point
(except for any Actions using `komodo.execute_terminal`, see [below](/docs/setup/version-upgrades#migrate-to-new-terminal-actions-methods) to fix this.)

### Move to public key authentication

If you want to keep the Core to Periphery connection direction, you can increase the security by
**moving from passkey authentication to public key authentication**.
If you want to reverse the agent connection, **skip this step**.

Navigate to the `Settings` page, at the top you will find the Core Public Key (starting with `MCow...`).
Copy this key and and **redeploy Periphery agents with updated configuration**:

```toml
## Accepted public keys to allow Core(s) to connect.
## Periphery gains knowledge of the Core public key through the noise handshake.
## If neither these nor passkeys provided, inbound connections will not be authenticated.
## Accepts Spki base64 DER directly and PEM file. Use `file:/path/to/core.pub` to load from file.
## Env: PERIPHERY_CORE_PUBLIC_KEYS
## Optional, no default.
core_public_keys = "<YOUR_CORE_PUBLIC_KEY>"
```

After confirming the connection still works, you can remove any legacy `passkey` configuration for both Core and Periphery as it is no longer needed.

### Reversing the agent connection

The Periphery agent is now able to establish an outbound connection to Komodo Core.
After updating to Komodo v2, **you can follow these steps to migrate to outbound connections
using public key authentication**. Note you must be an Admin user on Komodo.

1. **Periphery container only**. Ensure the automatically generated private keys are persisted by mounting to the `/config/keys` of the Periphery container, as noted above.
2. Navigate to `Settings / Onboarding` and create a new **Onboarding Key**. Save it for later.
3. Enable **Priviledged mode** on the new Onboarding Key.
4. **Redeploy Periphery agents with updated configuration:**

```toml
## The address of Komodo Core. Must be reachable from this host.
## If provided, Periphery will operate in outbound mode.
## Env: PERIPHERY_CORE_ADDRESS
## Default: None
core_address = "<YOUR_KOMODO_CORE_ADDRESS>" # Example: demo.komo.do, localhost:9120

## The Server this Periphery agent should "connect as".
## Systemd periphery can use the system hostname by setting `connect_as = "$(hostname)"`.
## Env: PERIPHERY_CONNECT_AS
## Default: None
connect_as = "<SERVER_NAME>"

## Make Onboarding Keys in Server settings.
## Not needed if connecting as Server that already exists.
## Env: PERIPHERY_ONBOARDING_KEY
onboarding_key = "<YOUR_ONBOARDING_KEY>"
```

Upon connecting, the Priviledged Onboarding Key will allow the existing Server's expected public key
to be updated, allowing the Periphery agent to connect.
In general when onboarding *new* Servers, Privilidged Mode is not needed.

### Migrate to new Terminal actions methods

