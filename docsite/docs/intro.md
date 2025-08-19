---
slug: /intro
---

# What is Komodo?

Komodo is a web app to provide structure for managing your servers, builds, deployments, and automated procedures.

With Komodo you can:

 - Connect all of your servers, alert on CPU usage, memory usage, and disk usage, and connect to shell sessions.
 - Create, start, stop, and restart Docker containers on the connected servers, view their status and logs, and connect to container shell.
 - Deploy docker compose stacks. The file can be defined in UI, or in a git repo, with auto deploy on git push.
 - Build application source into auto-versioned Docker images, auto built on webhook. Deploy single-use AWS instances for infinite capacity.
 - Manage repositories on connected servers, which can perform automation via scripting / webhooks.
 - Manage all your configuration / environment variables, with shared global variable and secret interpolation.
 - Keep a record of all the actions that are performed and by whom.

There is no limit to the number of servers you can connect, and there will never be. There is no limit to what API you can use for automation, and there never will be. No "business edition" here.

## Docker

Komodo is opinionated by design, and uses [docker](https://docs.docker.com/) as the container engine for building and deploying.

:::info
Komodo also supports [**podman**](https://podman.io/) instead of docker by utilizing the `podman` -> `docker` alias.
For Stack / docker compose support with podman, check out [**podman-compose**](https://github.com/containers/podman-compose). Thanks to `u/pup_kit` for checking this.
:::

## Architecture and Components

Komodo is composed of a single core and any amount of connected servers running the periphery application. 

### Core
Komodo Core is a web server hosting the Core API and browser UI. All user interaction with the connected servers flow through the Core.

### Periphery
Komodo Periphery is a small stateless web server that runs on all connected servers. It exposes an API called by Komodo Core to perform actions on the server, get system usage, and container status / logs. It is only intended to be reached from the core, and has an address whitelist to limit the IPs allowed to call this API.

## Core API

Komodo exposes powerful functionality over the Core's REST and Websocket API, enabling infrastructure engineers to manage their infrastructure programmatically. There is a [rust crate](https://crates.io/crates/komodo_client) and [npm package](https://www.npmjs.com/package/komodo_client) to simplify programmatic interaction with the API, but in general this can be accomplished using any programming language that can make REST requests. 

## Permissioning

Komodo is a system designed to be used by many users, whether they are developers, operations personnel, or administrators. The ability to affect an applications state is very powerful, so Komodo has a granular permissioning system to only provide this functionality to the intended users. The permissioning system is explained in detail in the [permissioning](/docs/resources/permissioning) section. 

User sign-on is possible using username / password, or with Oauth (Github and Google). See [Core Setup](./setup/index.mdx).