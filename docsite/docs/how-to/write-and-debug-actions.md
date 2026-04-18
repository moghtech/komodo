---
sidebar_label: Write and Debug Actions
---

# Write And Debug Actions

Actions are TypeScript scripts that run inside Komodo and call the Komodo API through a
pre-initialized `komodo` client. They are the right tool when a [Procedure](../automate/procedures.md)
is not flexible enough and you need scripting, branching, or API-driven iteration.

## Action Model

[Procedures and Actions](../automate/procedures.md) defines the Action resource and the basic
scripting model.

## Find The API Surface

Use these docs together:

- [Komodo API docs](../ecosystem/api.md) for the client surface and published packages
- [Procedures and Actions](../automate/procedures.md) for examples
- [`docs.rs` API reference](https://docs.rs/komodo_client/latest/komodo_client/api/index.html) for
  exact request and response types
- [Terminals](../terminals.md) when the action needs remote shell access

If you are looking for the fields on a specific request body such as
`komodo.execute_server_terminal`, the `docs.rs` struct page shows the request and response types.

## First Script Shape

A first Action can do one of these:

- read a resource list
- update one variable
- run one execution against one known resource

That keeps the first failure mode obvious. After the first read, write, or execute call works, add
loops, filters, or terminal integration.

## A Minimal Pattern

```ts
const deployments = await komodo.read("ListDeployments", {});

for (const deployment of deployments) {
  console.log(`Found ${deployment.name}`);
}
```

Then move to a write or execute call:

```ts
await komodo.execute("RestartDeployment", {
  deployment: "api",
});
```

## Use Terminals Deliberately

When you need host-side command execution, Actions can call terminal APIs such as
`execute_server_terminal`. That path combines server selection, terminal naming, recreate behavior,
and command initialization in one call.

Read [Terminals](../terminals.md) before relying on terminal execution in an Action. The terminal
page explains the different targets and the lifecycle of named terminal sessions.

## Debugging Approach

Use a tight debugging loop:

1. test with one resource
2. log intermediate values with `console.log`
3. verify the API call shape against `docs.rs`
4. only then generalize to tags, patterns, or multiple resources

If an Action is mostly a fixed series of known executions, move it back into a
[Procedure](../automate/procedures.md). Use an Action when the script itself is making the
decision.

## Related Pages

- [Procedures and Actions](../automate/procedures.md)
- [Terminals](../terminals.md)
- [Komodo API](../ecosystem/api.md)
- [Webhooks](../automate/webhooks.md)
