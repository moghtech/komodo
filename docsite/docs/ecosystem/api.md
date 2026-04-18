# API and Clients

Komodo Core exposes an RPC-style HTTP API for reading state, writing configuration, and executing
actions. The same API is available through typed clients in Rust and TypeScript.

## Choose The Interface

- Use the [Komodo CLI](./cli.mdx) when the task is interactive or shell-driven.
- Use the Rust or TypeScript client when the integration should stay typed.
- Use direct HTTP when another tool already owns the request flow.

## API Reference

The authoritative API reference is published on
[docs.rs](https://docs.rs/komodo_client/latest/komodo_client/api/index.html).

## Rust Client

The Rust client is published to crates.io as
[`komodo_client`](https://crates.io/crates/komodo_client).

```rust
let komodo = KomodoClient::new("https://demo.komo.do", "your_key", "your_secret")
  .with_healthcheck()
  .await?;

let stacks = komodo.read(ListStacks::default()).await?;

let update = komodo
  .execute(DeployStack {
    stack: stacks[0].name.clone(),
    stop_time: None,
  })
  .await?;
```

## TypeScript Client

The TypeScript client is published to NPM as
[`komodo_client`](https://www.npmjs.com/package/komodo_client).

```ts
import { KomodoClient, Types } from "komodo_client";

const komodo = KomodoClient("https://demo.komo.do", {
  type: "api-key",
  params: {
    key: "your_key",
    secret: "your secret",
  },
});

// Inferred as Types.StackListItem[]
const stacks = await komodo.read("ListStacks", {});

// Inferred as Types.Update
const update = await komodo.execute("DeployStack", {
  stack: stacks[0].name,
});
```

## Direct HTTP Use

If another system already controls the request flow, you can call the API over HTTP directly. The
typed clients are still useful as the reference for method names, request bodies, and response
shapes.

## API Keys

API keys authenticate requests through request headers. They are the normal non-interactive auth
path for scripts, integrations, CI jobs, and client libraries.

There are two main cases:

- users can create API keys for themselves
- admins can create service users and API keys for those service users

The secret is only shown when the key is created. Komodo stores only the hashed secret afterward.

The same API-key auth model also applies to websocket login for update and terminal connections.

Use a service user when the automation should have its own identity and permissions instead of
borrowing a human user's credentials. See [Permissioning](../configuration/permissioning.md).

## Related Pages

- [Komodo CLI](./cli.mdx)
- [Write And Debug Actions](../how-to/write-and-debug-actions.md)
- [Procedures and Actions](../automate/procedures.md)
- [Connection Model](../connection-model.md)
- [Permissioning](../configuration/permissioning.md)
