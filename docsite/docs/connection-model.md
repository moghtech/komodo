# Connection Model

Komodo uses an authenticated websocket connection between [Core](./core.md) and
[Periphery](./periphery.md). That connection model determines how servers are added, how trust is
bootstrapped, and which component needs to be reachable on the network.

## Core And Periphery

[Core](./core.md) is the control plane. [Periphery](./periphery.md) is the host-side agent. A
managed [Server](./server.md) exists when Core has a server record and a Periphery agent can
connect as that server.

That connection also defines:

- which host can execute work
- which key material Core trusts for that server
- whether Core needs to reach Periphery, or Periphery needs to reach Core

## Two Connection Shapes

### Outbound Periphery To Core

Periphery initiates the connection to Core. This shape fits the bundled local install, hosts behind
NAT, and environments where Periphery should not expose a public listener.

This shape fits when:

- Core is reachable from the host running Periphery
- Periphery should not expose its own listener publicly
- a private network, overlay network, or public Core endpoint already exists

### Inbound Core To Periphery

Core initiates the connection to a reachable Periphery listener. This requires Periphery to expose
an address and port that Core can reach, directly or through a reverse proxy.

This shape fits when:

- Core already has a stable path to the target host
- you want Core to connect to Periphery directly
- Periphery is running with a host-managed address that Core can reach consistently

If Periphery must be reachable on the network, treat that as an explicit network design choice. See
[Reverse Proxy And Periphery Access](./how-to/reverse-proxy-and-periphery-access.md).

## Onboarding And Server Identity

[Onboarding Keys](./setup/onboarding-keys.md) bootstrap trust and create or update the server
record in Core. On first connection, Periphery generates its own key pair and Core stores the
public key it expects for that server.

Use onboarding for a new server because it:

- Core creates the server record
- trust is bootstrapped in one step
- Periphery can connect without manual key distribution

For an existing server, Periphery can reconnect without a new onboarding key as long as:

- the server record already exists in Core
- `connect_as` matches the existing Server name or id
- the Periphery key material is persisted so the agent keeps the same identity

See [Connect Periphery Without An Onboarding Key](./how-to/connect-periphery-without-onboarding-key.md)
for the manual path.

## Network Placement

The network design follows from the connection shape.

When possible, keep Core and Periphery on a private network, overlay network, or another path that
does not require public Periphery exposure. If Core is public but Periphery should not be, restrict
the Core websocket endpoint and keep the Periphery listener private.

Public Periphery exposure is the exception, not the default. If you need it, bind Periphery
carefully, restrict source addresses, and keep TLS enabled.

## Operational Consequences

Several setup and operations questions reduce to the same connection model:

- adding a new server
- reconnecting an existing server
- rotating keys
- deciding whether Periphery should call Core or vice versa
- deciding which endpoint should sit behind a reverse proxy

Those are network and trust questions before they are installation details.

## Related Pages

- [What Is Komodo](./intro.md)
- [Core](./core.md)
- [Periphery](./periphery.md)
- [Server](./server.md)
- [Setup](./setup)
- [Connect More Servers](./setup/connect-servers.mdx)
- [Onboarding Keys](./setup/onboarding-keys.md)
- [Add Another Server](./how-to/add-another-server.md)
- [Connect Periphery Without An Onboarding Key](./how-to/connect-periphery-without-onboarding-key.md)
- [Reverse Proxy And Periphery Access](./how-to/reverse-proxy-and-periphery-access.md)
