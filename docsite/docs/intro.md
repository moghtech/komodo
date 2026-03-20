---
slug: /intro
---

# What is Komodo?

Komodo is a web application for managing servers, builds, deployments, and automated procedures.

With Komodo you can:

- **Connect all of your servers**, alert on CPU usage, memory usage, and disk usage, and connect to shell sessions.
- **Create, start, stop, and restart Docker containers** on the connected servers, view their status and logs, and connect to container shell.
- **Deploy docker compose stacks.** The file can be defined in UI, or in a git repo, with auto deploy on git push.
- **Build application source into auto-versioned Docker images**, triggered automatically on webhook. Integrates with AWS EC2 spot instances for scalable build capacity.
- **Manage repositories on connected servers**, which can perform automation via scripting / webhooks.
- **Manage all your configuration / environment variables**, with shared global variable and secret interpolation.
- **Keep a record of all the changes** that are made and by whom.

There is no limit to the number of servers you can connect, and there will never be.
There is no limit to what API you can use for automation, and there never will be.

## Docker

Komodo is opinionated by design, and uses [docker](https://docs.docker.com/) as the container engine for building and deploying.

:::info
Komodo also can support [**podman**](https://podman.io/) instead of docker by utilizing the `podman` -> `docker` alias.
:::

## Architecture and Components

Komodo is composed of a single Core and any amount of connected servers running the Periphery agent.

### Core

Komodo Core is a **web server hosting the Core API and browser UI**. All user interaction with the connected servers flow through the Core.

### Periphery

Komodo Periphery is a **small stateless agent** that runs on all connected servers. It exposes an API called by Komodo Core to perform actions on the server, get system usage, and container status / logs.
In order to communicate with Core, it can be configured to **either initiate outbound connections or accept inbound ones**, whichever is simplest for your environment.

## Core API

Komodo exposes powerful functionality over **Core's REST and Websocket API**, enabling engineers to **implement complex automations easily** and manage their infrastructure programmatically. There is the [**Komodo CLI**](./ecosystem/cli.mdx), [**Rust crate**](https://crates.io/crates/komodo_client), and [**Npm package**](https://www.npmjs.com/package/komodo_client) to simplify programmatic interaction with the API, but this can also be accomplished just [using `curl`](https://docs.rs/komodo_client/latest/komodo_client/api/index.html#curl-example).

## Permissioning

Komodo is a system **designed to be used by many users working together**, whether they are developers, operations personnel, or administrators. The ability to affect an applications state is very powerful, so **Komodo has a granular permissioning system** to only provide this functionality to the intended users. The permissioning system is explained in more detail in the [permissioning](/docs/resources/permissioning) section.

User sign-on is possible using **username / password**, or with **OAuth (GitHub, Google, and generic OIDC)**. See [Core Setup](./setup/index.mdx).
