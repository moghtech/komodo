---
sidebar_label: Add Another Server
---

# Add Another Server

Once Core is up and the first local or bundled server is visible, add the next machine with an
[onboarding key](../setup/onboarding-keys.md).

To add a new host:

1. create an onboarding key in `Settings > Onboarding`
2. install Periphery on the new host
3. pass the onboarding key during install
4. confirm the new Server reaches `OK`

## Create The Onboarding Key

In the Komodo UI, open `Settings > Onboarding` and create a new onboarding key.

Save the key when it is shown. The installer or container configuration on the target host will
need it for first connection. See [Onboarding Keys](../setup/onboarding-keys.md) for the key
options and reconnect behavior.

## Choose The Periphery Install Shape

[Connect More Servers](../setup/connect-servers.mdx) covers the install details. The main choice
here is:

- host-managed Periphery for the clearest host and filesystem model
- containerized Periphery when that matches the host better or `systemd` is not available

Do not combine a host-managed Periphery and the bundled `periphery` service for the same host.

## Install Periphery On The New Host

For the host-managed path, run the installer script with:

- Core address
- server name
- onboarding key

For the containerized path, use the published Periphery Compose file and set:

- `PERIPHERY_CORE_ADDRESS`
- `PERIPHERY_CONNECT_AS`
- `PERIPHERY_ONBOARDING_KEY`

## Confirm The New Server

After the agent starts, check the Server in the Komodo UI:

- connection status should reach `OK`
- the Server name should match what you expect
- host-side resources should appear from that host, not the machine running Core

If the connection does not come up, common causes are:

- wrong Core address
- wrong or expired onboarding key
- duplicate Periphery paths for the same host
- a path or mount mismatch in containerized Periphery

## Existing Server Versus New Server

If the server already exists in Komodo and you are reconnecting it, you may not need a new
onboarding key. See [Connect Periphery Without An Onboarding Key](./connect-periphery-without-onboarding-key.md).

## Related Pages

- [Connect More Servers](../setup/connect-servers.mdx)
- [Periphery](../periphery.md)
- [Server](../server.md)
- [Connect Periphery Without An Onboarding Key](./connect-periphery-without-onboarding-key.md)
