# API and Clients

Komodo Core exposes an RPC-like HTTP API to read data, write configuration, and execute actions.
There are typesafe clients available in
[**Rust**](/docs/ecosystem/api#rust-client) and [**Typescript**](/docs/ecosystem/api#typescript-client).

The full API documentation is [**available here**](https://docs.rs/komodo_client/latest/komodo_client/api/index.html).

## Rust Client

The Rust client is published to crates.io at [komodo_client](https://crates.io/crates/komodo_client).

```rust
let komodo = KomodoClient::new("https://demo.komo.do", "your_key", "your_secret")
  .with_healthcheck()
  .await?;

let stacks = komodo.read(ListStacks::default()).await?;

let update = komodo
  .execute(DeployStack {
    stack: stacks[0].name.clone(),
    stop_time: None
  })
  .await?;
```

## Typescript Client

The Typescript client is published to NPM at [komodo_client](https://www.npmjs.com/package/komodo_client).

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
