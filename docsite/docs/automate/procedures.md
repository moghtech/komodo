# Procedures and Actions

Komodo offers `Procedure` and `Action` resources for orchestrating multi-resource workflows.

## Procedures

A Procedure composes multiple executions (like `RunBuild`, `DeployStack`, `Deploy`) into a series of **Stages**. Executions within a stage run **in parallel**; stages run **sequentially**. The Procedure waits for all executions in a stage to complete before moving to the next.

```toml
[[procedure]]
name = "build-and-deploy"
description = "Builds the app, then deploys both instances"

[[procedure.config.stage]]
name = "Build"
executions = [
  { execution.type = "RunBuild", execution.params.build = "my-app" },
]

[[procedure.config.stage]]
name = "Deploy"
executions = [
  { execution.type = "Deploy", execution.params.deployment = "my-app-01" },
  { execution.type = "Deploy", execution.params.deployment = "my-app-02" },
]
```

### Config fields

| Field | Description | Default |
|---|---|---|
| `schedule` | Cron-style schedule string (e.g. `"Every day at 03:00"`). | — |
| `config.stage[].name` | Display name for the stage. | — |
| `config.stage[].enabled` | Whether the stage is active. | `true` |
| `config.stage[].executions` | List of executions to run in parallel within the stage. | `[]` |

### Batch Executions

Many executions have a `Batch` variant (e.g. [**BatchDeployStackIfChanged**](https://docs.rs/komodo_client/latest/komodo_client/api/execute/struct.BatchDeployStackIfChanged.html)) that matches multiple resources by name using [wildcard](https://docs.rs/wildcard/latest/wildcard) and [regex](https://docs.rs/regex/latest/regex) syntax.

```toml
[[procedure.config.stage]]
name = "Deploy matching stacks"
executions = [
  { execution.type = "BatchDeployStackIfChanged", execution.params.pattern = "foo-* , \\^bar-.*$\\" },
]
```

## Actions

Actions let you write Typescript scripts that call the Komodo API. A pre-initialized `komodo` client is available — no API keys needed. The editor provides type-aware suggestions and inline docs.

```ts
const VERSION = "1.16.5";
const BRANCH = "dev/" + VERSION;
const APPS = ["core", "periphery"];
const ARCHS = ["x86", "aarch64"];

await komodo.write("UpdateVariableValue", {
  name: "KOMODO_DEV_VERSION",
  value: VERSION,
});

for (const app of APPS) {
  for (const arch of ARCHS) {
    const name = `komodo-${app}-${arch}-dev`;
    await komodo.write("UpdateBuild", {
      id: name,
      config: { version: VERSION as any, branch: BRANCH },
    });
    console.log(`Updated Build ${name}`);
  }
}
```

The Typescript client is also [published on NPM](https://www.npmjs.com/package/komodo_client).
