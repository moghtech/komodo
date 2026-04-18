# Terminals

Komodo provides browser-based terminal sessions for servers and containers. Sessions are persistent,
support multiple simultaneous connections, and can also be used from the CLI or Actions.

## Server Terminals

Open a shell directly on a connected server. The default command is `bash` and can be configured
per Periphery with `default_terminal_command`.

## Container Terminals

Connect to a running container in two modes:

- **Exec** (default) runs a new command inside the container with `docker exec`.
- **Attach** attaches to the container's main process with `docker attach`.

Container terminals are available on **Deployments**, **Stack services**, and any container visible
on a server.

## Multiple Sessions

You can create multiple named terminal sessions on the same resource. Each session has its own
independent PTY process and output history.

- Terminal names must be unique within a target.
- Multiple users can connect to the same terminal session simultaneously. Output is broadcast to
  all connected clients.
- Sessions persist until explicitly deleted or Periphery restarts.

## Terminal History

Each terminal maintains a rolling 1 MiB output buffer. When you reconnect to an existing session,
the history is replayed so you can see previous output.

## CLI

Terminal sessions can also be accessed from the command line using the
[Komodo CLI](./ecosystem/cli.mdx#terminals).

- `km ssh <server>` — open a shell on a server
- `km exec <container> <shell>` — exec into a container
- `km attach <container>` — attach to a container's main process

Press **Alt+Q** to disconnect from any CLI terminal session while the session itself stays running.

## Execute Terminal

The `execute_terminal` API method allows you to run a command on a terminal and stream the output
back over HTTP. This is useful for:

- **Actions**: TypeScript scripts can call `execute_terminal` on the Komodo client to run commands
  on any server or container and process the output programmatically.
- **Automation**: integrate terminal command execution into external tools via the REST API.

The TypeScript client provides convenience methods for each target type. All methods accept
optional `callbacks` with `onLine` and `onFinish`.

```ts
// Server terminal
await komodo.execute_server_terminal({
  server: "my-server",
  terminal: "automation",
  command: "df -h",
  init: { command: "bash", recreate: "DifferentCommand" },
}, {
  onLine: (line) => console.log(line),
  onFinish: (code) => console.log("Exit code:", code),
});

// Container terminal
await komodo.execute_container_terminal({
  server: "my-server",
  container: "my-container",
  terminal: "debug",
  command: "cat /var/log/errors.log",
  init: { command: "sh", mode: "Exec", recreate: "Never" },
});

// Stack service terminal
await komodo.execute_stack_service_terminal({
  stack: "my-stack",
  service: "web",
  terminal: "debug",
  command: "nginx -t",
  init: { command: "sh" },
});

// Deployment terminal
await komodo.execute_deployment_terminal({
  deployment: "my-deployment",
  terminal: "check",
  command: "node --version",
  init: { command: "sh", recreate: "Always" },
});
```

## Periphery Configuration

Terminal behavior can be configured in the Periphery config file:

| Setting | Description | Default |
| --- | --- | --- |
| `default_terminal_command` | Default shell command for new server terminals. | `bash` |
| `disable_terminals` | Disable server terminal sessions. | `false` |
| `disable_container_terminals` | Disable container terminal sessions. | `false` |

## Related Pages

- [Periphery](./periphery.md)
- [Server](./server.md)
- [Permissioning](./configuration/permissioning.md)
- [Procedures and Actions](./automate/procedures.md)
