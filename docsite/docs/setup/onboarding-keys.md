---
sidebar_label: Onboarding Keys
---

# Onboarding Keys

An onboarding key bootstraps trust between Komodo Core and a new Periphery agent. New Server
connections use it to establish the first trusted relationship.

Core stores the public side of the onboarding key. The private onboarding key is returned once when
the key is created and is not available again afterward.

## What An Onboarding Key Does

An onboarding key handles the first trust step:

- Core creates or updates the Server record
- Periphery connects with the onboarding key
- Periphery generates its own long-term keypair
- Core stores the Periphery public key it should trust for that Server

After first connection, the onboarding key is no longer needed for that server. Ongoing trust uses
the Periphery keypair stored on the host and the public key stored in Core.

## First Connection Lifecycle

A first connection through onboarding looks like this:

1. create an onboarding key in `Settings > Onboarding`
2. install Periphery on the target host
3. pass the onboarding key during the first connection
4. confirm the Server reaches `OK`

See [Connect More Servers](./connect-servers.mdx) for the installation path and
[Connection Model](../connection-model.md) for the trust model behind it.

## Key Options

Onboarding keys support a few important options:

- `enabled`: disable the key when it should not be used
- `expires`: set an expiry time, or leave it at `0` for no expiry
- `name`: give the key a management label in the UI
- `tags`: apply default tags to Servers created with the key
- `copy_server`: initialize the new Server from an existing Server config
- `create_builder`: create a Builder for the new Server
- `onboarded`: track which Server ids were created with the key

Those options make onboarding keys useful for fleet setup as well as one-off installs.

## Privileged Keys

The `privileged` option expands what the key can do. A privileged onboarding key can:

1. enable a disabled Server
2. remove a Server `address` so it can switch to a Periphery -> Core connection
3. update the public keys stored for an existing Server

That puts privileged keys in reconnect and recovery workflows as well as first install.

## New Server Versus Existing Server

New Servers use onboarding keys.

For an existing Server, a new onboarding key is not always required. Reconnection can work without
it when:

- the Server record already exists in Core
- `connect_as` matches the existing Server name or id
- the Periphery private key has been preserved on the host

See [Connect Periphery Without An Onboarding Key](../how-to/connect-periphery-without-onboarding-key.md)
for that path.

## Related Pages

- [Connect More Servers](./connect-servers.mdx)
- [Connection Model](../connection-model.md)
- [Add Another Server](../how-to/add-another-server.md)
- [Connect Periphery Without Onboarding](../how-to/connect-periphery-without-onboarding-key.md)
