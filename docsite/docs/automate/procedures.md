# Procedures and Actions

Komodo offers `Procedure` and `Action` resources for multi-step automation.

Use a `Procedure` when the workflow is a sequence of built-in executions such as `RunBuild`,
`DeployStack`, `Deploy`, or `PullRepo`.

Use an `Action` when the workflow needs TypeScript, branching, iteration, or direct API calls.

Both resources can also be run on a [schedule](./schedules.md) or triggered by
[webhooks](./webhooks.md).

## Choose Between Procedures And Actions

A `Procedure` runs built-in executions across resources in staged order. An `Action` runs
TypeScript against the Komodo API when the workflow needs branching, iteration, or logic that does
not fit well into staged executions.

## Procedures

A Procedure composes multiple executions into a series of **Stages**. Executions within a stage run
in parallel. Stages run sequentially. Komodo waits for the whole stage to finish before moving to
the next one.

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

### Config Fields

| Field | Description | Default |
| --- | --- | --- |
| `config.stage[].name` | Display name for the stage. | — |
| `config.stage[].enabled` | Whether the stage is active. | `true` |
| `config.stage[].executions` | Executions to run in parallel within the stage. | `[]` |
| `schedule` | Schedule expression. See [Schedules](./schedules.md). | `""` |
| `schedule_format` | `English` or `Cron`. | `English` |
| `schedule_enabled` | Whether the schedule is active. | `true` |

### Batch Executions

Many executions also have a `Batch` variant. These match multiple resources by name using
[wildcard](https://docs.rs/wildcard/latest/wildcard) and
[regex](https://docs.rs/regex/latest/regex) syntax.

For example,
[BatchDeployStackIfChanged](https://docs.rs/komodo_client/latest/komodo_client/api/execute/struct.BatchDeployStackIfChanged.html)
can deploy several stacks that match one pattern:

```toml
[[procedure.config.stage]]
name = "Deploy matching stacks"
executions = [
  {
    execution.type = "BatchDeployStackIfChanged",
    execution.params.pattern = "foo-* , \\^bar-.*$\\",
  },
]
```

## Actions

Actions let you write TypeScript scripts that call the Komodo API from inside Komodo. A
pre-initialized `komodo` client is available, so the script does not need its own API-key setup.
The in-UI editor provides type-aware suggestions and inline documentation.

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

The TypeScript client is also [published on NPM](https://www.npmjs.com/package/komodo_client).

### Action Examples

#### Restart All Deployments Matching Tags

```ts
const deployments = await komodo.read("ListDeployments", {
  query: { tags: ["backend"] },
});

for (const deployment of deployments) {
  await komodo.execute("RestartDeployment", {
    deployment: deployment.name,
  });
  console.log(`Restarted ${deployment.name}`);
}
```

#### Run A Command On A Server Terminal

```ts
await komodo.execute_server_terminal({
  server: "server-prod",
  command: "df -h",
  init: { command: "bash" },
}, {
  onLine: (line) => console.log(line),
  onFinish: (code) => console.log("Exit code:", code),
});
```

#### Scale A Deployment Based On Time Of Day

```ts
const hour = new Date().getHours();
const replicas = hour >= 9 && hour <= 17 ? "4" : "1";

await komodo.write("UpdateDeployment", {
  id: "api-server",
  config: {
    extra_args: [`--replicas=${replicas}`],
  },
});

await komodo.execute("Deploy", { deployment: "api-server" });
console.log(`Scaled api-server to ${replicas} replicas`);
```

## Related Pages

- [Write And Debug Actions](../how-to/write-and-debug-actions.md)
- [Webhooks](./webhooks.md)
- [Schedules](./schedules.md)
- [Terminals](../terminals.md)
