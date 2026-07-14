---
sidebar_label: Connect Periphery Without Onboarding
---

# Connect Periphery Without An Onboarding Key

Onboarding keys add new Server records and establish first trust between Core and Periphery. Use
that path for first install and for new hosts.

Use the no-onboarding path when:

- the Server already exists in Komodo
- you are reconnecting or replacing an existing Periphery agent
- you want to manage the Core and Periphery keys explicitly

## Outbound Mode Against An Existing Server

For the no-onboarding path, create or keep the Server entry in Komodo first, then run
Periphery in outbound mode with:

- `PERIPHERY_CORE_ADDRESS`
- `PERIPHERY_CONNECT_AS`
- no `PERIPHERY_ONBOARDING_KEY`

The `connect_as` value must match the existing Server name or id. This is the path the Periphery
config comments describe as `Not needed if connecting as Server that already exists.`

## Persist The Periphery Keys

Without an onboarding flow creating new server state for you, stable key material matters more.
Persist the Periphery private key so the agent keeps the same identity across restarts:

- container installs: persist `/config/keys`
- host installs: persist `${root_directory}/keys`

If you replace the key material, Core will see a different agent identity and the connection may be
rejected until the expected public key is updated.

## Inbound Mode

For inbound Core -> Periphery connections, Periphery needs to trust Core's public key through
`PERIPHERY_CORE_PUBLIC_KEYS`, and Core needs to trust the connecting Periphery key for that server.

This path is more explicit and more manual than onboarding. Use it when you already have stable
keys and want to connect a known Core to a known Periphery without creating or distributing a
reusable onboarding token.

## New Server Versus Existing Server

If the server does not already exist in Komodo, use an onboarding key. The no-onboarding flow fits
reconnect and manual-key-management cases where the Server record and trust material already exist.

The cryptography stays the same. The operator takes on more key management and connection setup.

## When To Prefer An Onboarding Key Anyway

Use an onboarding key when:

- you are adding a new server
- you want Komodo to create the server record for you
- you want the least manual PKI work
- you need a privileged onboarding key to update the expected public key on an existing server

## Related Pages

- [Add Another Server](./add-another-server.md)
- [Connect More Servers](../setup/connect-servers.mdx)
- [Periphery](../periphery.md)
- [Server](../server.md)
- [Komodo v2 Release Notes](../releases/v2.0.0.md)
