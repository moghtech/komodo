# Procedures and Actions

Komodo offers `Procedure` and `Action` resources for orchestrating multi-resource workflows. Both can be run on a **[schedule](schedules)**

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
| `config.stage[].name` | Display name for the stage. | — |
| `config.stage[].enabled` | Whether the stage is active. | `true` |
| `config.stage[].executions` | List of executions to run in parallel within the stage. | `[]` |
| `schedule` | Schedule expression. See [Scheduling](scheduling). | `""` |
| `schedule_format` | `English` or `Cron`. | `English` |
| `schedule_enabled` | Whether the schedule is active. | `true` |

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

Actions let you write Typescript scripts that call the Komodo API. A pre-initialized `komodo` client is available — no API key configuration needed.
The in UI editor provides type-aware suggestions and inline documentation.

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

### Action examples

#### Restart all deployments matching tags

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

#### Run a command on a server terminal

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

#### Scale a deployment based on time of day

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
