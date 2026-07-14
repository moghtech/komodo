# Development

Komodo can be developed from a devcontainer, from a local host toolchain with Docker for the
database, or through the compose-based development stack.

## Development Paths

There are three development paths:

- **Devcontainer**: full environment in a container, with the database started through
  `.devcontainer/dev.compose.yaml`
- **Local Rust and UI, Docker database**: run Core, Periphery, and the UI on the host, and run the
  database in Docker
- **Compose-based dev stack**: rebuild and start the development Compose stack for integrated local
  testing

## Dependencies

Running Komodo from [source](https://github.com/moghtech/komodo) needs either Docker with the
included devcontainer, or a local toolchain with these pieces installed:

- **Backend**
  - [Rust](https://www.rust-lang.org/) stable via [rustup](https://rustup.rs/)
  - [MongoDB](https://www.mongodb.com/) or [FerretDB](https://www.ferretdb.com/), either local or
    in Docker
  - on Debian or Ubuntu: `build-essential`, `pkg-config`, and `libssl-dev`
- **Web UI and generated client**
  - [Node](https://nodejs.org/en) 18.18 or newer
  - [Yarn](https://yarnpkg.com/) via `corepack enable` or another local install
  - [Deno](https://deno.com/) 2.0.2 or newer
  - [`typeshare-cli`](https://github.com/1Password/typeshare)

## Project Task Runner

This repo uses [runnables-cli](https://github.com/mbecker20/runnables-cli) for common project
tasks defined in `runfile.toml`.

The docs below use `run ...` commands from that file, such as:

- `run dev-core`
- `run dev-periphery`
- `run dev-ui`
- `run gen-client`
- `run dev-compose-build`
- `run dev-compose-exposed`
- `run dev-docsite`

If you do not want to use `runnables-cli`, the underlying commands are in `runfile.toml`.

## Devcontainer

The repo includes `.devcontainer/devcontainer.json` and `.devcontainer/dev.compose.yaml`.

This path:

- starts the database service from `.devcontainer/dev.compose.yaml`
- mounts the repo into `/workspace`
- forwards ports `5173` and `9120`
- runs `./.devcontainer/postCreate.sh` after the container is created

Inside the container, VS Code tasks are already defined for the common flow:

1. run `Init`
2. run `Run Komodo`

There are also separate tasks for Core, Periphery, and the UI if you only need one part of the
stack.

## Compose-Based Development Stack

Use the Compose-based path when you want to rebuild and run the integrated development stack.

The main commands are:

```bash
run dev-compose-build
run dev-compose-exposed
```

This rebuilds the development images and starts the stack with the UI exposed on `localhost:9120`.

Any source change that should affect the Compose-based stack requires rebuilding and restarting the
relevant services.

## Local Development

You can also run Core, Periphery, and the UI directly on the host and use Docker only for the
database.

### Initial Setup

Create the local config directories:

```bash
mkdir -p .dev/keys .dev/periphery
```

Add `.dev/core.config.toml`:

```toml
host = "http://localhost:9120"
private_key = "file:.dev/keys/core.key"
local_auth = true
enable_new_users = true
jwt_secret = "a_random_secret"
first_server_address = "http://localhost:8120"
cors_allowed_origins = ["http://localhost:5173"]
cors_allow_credentials = true
session_allow_cross_site = true

database.address = "localhost:27017"
database.username = "komodo"
database.password = "komodo"
```

Add `.dev/periphery.config.toml`:

```toml
ssl_enabled = false
root_directory = ".dev/periphery"
```

Add `ui/.env.development`:

```bash
VITE_KOMODO_HOST=http://localhost:9120
```

Install the development tools and link the generated TypeScript client into the UI:

```bash
rustup update
cargo install typeshare-cli runnables-cli
run link-client
```

### Start The Database

Start MongoDB in Docker:

```bash
docker run -d --name komodo-mongo \
  -p 27017:27017 \
  -v komodo-mongo-data:/data/db \
  -v komodo-mongo-config:/data/configdb \
  -e MONGO_INITDB_ROOT_USERNAME=komodo \
  -e MONGO_INITDB_ROOT_PASSWORD=komodo \
  mongo
```

### Start Core, Periphery, And UI

Run these in separate terminals:

```bash
run dev-core
```

```bash
run dev-periphery
```

```bash
run dev-ui
```

Then open `http://localhost:5173`.

### Regenerate The TypeScript Client

After API changes, rebuild the generated client:

```bash
run gen-client
```

This regenerates the types and rebuilds the TypeScript package used by the UI.

## Docsite Development

Use:

```bash
run dev-docsite
```

That starts the Docusaurus docs site in development mode with live reload for changes under
`docsite/`.

## Related Pages

- [API and Clients](./api.md)
- [Writing Guidelines](./writing-guidelines.md)
