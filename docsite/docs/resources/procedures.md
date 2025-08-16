# Procedures and Actions

For orchestrations involving multiple resources and executions,
Komodo offers the `Procedure` and `Action` resource types.

## Procedures

`Procedures` are compositions of many executions, such as `RunBuild` and `DeployStack`.
The executions are grouped into a series of `Stages`, where each `Stage` contains one or more executions
to run **_all at once_**. The Procedure will wait until all of the executions in a `Stage` are complete before moving
on to the next stage. In short, the executions in a `Stage` are run **_in parallel_**, and the stages themselves are
executed **_sequentially_**.

### Batch Executions

Many executions have a `Batch` version you can select, for example [**BatchDeployStackIfChanged**](https://docs.rs/komodo_client/latest/komodo_client/api/execute/struct.BatchDeployStackIfChanged.html). With this, you can match multiple Stacks by name
using [**wildcard syntax**](https://docs.rs/wildcard/latest/wildcard) and [**regex**](https://docs.rs/regex/latest/regex).

### TOML Example

Like all Resources, `Procedures` have a TOML representation, and can be managed in `ResourceSyncs`.

```toml
[[procedure]]
name = "pull-deploy"
description = "Pulls stack-repo, deploys stacks"

[[procedure.config.stage]]
name = "Pull Repo"
executions = [
  { execution.type = "PullRepo", execution.params.pattern = "stack-repo" },
]

[[procedure.config.stage]]
name = "Deploy if changed"
executions = [
  # Uses the Batch version, witch matches many stacks by pattern
  # This one matches all stacks prefixed with `foo-` (wildcard) and `bar-` (regex).
  { execution.type = "BatchDeployStackIfChanged", execution.params.pattern = "foo-* , \\^bar-.*$\\" },
]
```

## Actions

`Actions` give users the power of Typescript to write calls to the Komodo API.

For example, an `Action` script like this will align the versions and branches of many `Builds`.

```ts
const VERSION = "1.16.5";
const BRANCH = "dev/" + VERSION;
const APPS = ["core", "periphery"];
const ARCHS = ["x86", "aarch64"];

await komodo.write("UpdateVariableValue", {
  name: "KOMODO_DEV_VERSION",
  value: VERSION,
});
console.log("Updated KOMODO_DEV_VERSION to " + VERSION);

for (const app of APPS) {
  for (const arch of ARCHS) {
    const name = `komodo-${app}-${arch}-dev`;
    await komodo.write("UpdateBuild", {
      id: name,
      config: {
        version: VERSION as any,
        branch: BRANCH,
      },
    });
    console.log(
      `Updated Build ${name} to version ${VERSION} and branch ${BRANCH}`,
    );
  }
}

for (const arch of ARCHS) {
  const name = `periphery-bin-${arch}-dev`;
  await komodo.write("UpdateRepo", {
    id: name,
    config: {
      branch: BRANCH,
    },
  });
  console.log(`Updated Repo ${name} to branch ${BRANCH}`);
}
```