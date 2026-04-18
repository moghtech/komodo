# Development

If you are looking to contribute to Komodo, this page is a launching point for setting up your Komodo development environment.

## Dependencies

Running Komodo from [source](https://github.com/moghtech/komodo) requires either [Docker](https://www.docker.com/) (and can use the included [devcontainer](https://code.visualstudio.com/docs/devcontainers/containers)), or can have the development dependencies installed locally:

* Backend (Core / Periphery APIs)
    * [Rust](https://www.rust-lang.org/) stable via [rustup installer](https://rustup.rs/)
    * [MongoDB](https://www.mongodb.com/) or [FerretDB](https://www.ferretdb.com/) available locally.
    * On Debian/Ubuntu: `apt install build-essential pkg-config libssl-dev` required to build the rust source.
* Web UI
    * [Node](https://nodejs.org/en) >= 18.18 + NPM
        * [Yarn](https://yarnpkg.com/) - (Tip: use `corepack enable` after installing `node` to use `yarn`)
    * [typeshare](https://github.com/1password/typeshare)
    * [Deno](https://deno.com/) >= 2.0.2

### runnables-cli

[mbecker20/runnables-cli](https://github.com/mbecker20/runnables-cli) can be used as a convience CLI for running common project tasks found in `runfile.toml`. Otherwise, you can create your own project tasks by references the `cmd`s found in `runfile.toml`. All instructions below will use runnables-cli v1.3.7+.

## Docker

After making changes to the project, run `run dev-compose-build` to rebuild Komodo and then `run dev-compose-exposed` to start a Komodo container with the UI accessible at `localhost:9120`. Any changes made to source files will require re-running the `dev-compose-build` and `dev-compose-exposed` commands.

## Devcontainer

Use the included `.devcontainer.json` with VSCode or other compatible IDE to stand-up a full environment, including database, with one click.

[VSCode Tasks](https://code.visualstudio.com/Docs/editor/tasks) are provided for building and running Komodo.

After opening the repository with the devcontainer run the task `Init` to build the ui/backend. Then, the task `Run Komodo` can be used to run ui/backend. Other tasks for rebuilding/running just one component of the stack (Core API, Periphery API, UI) are also provided.

## Local

You can also run the components locally, using Docker only for the database.

### Initial One-time Setup

Create the local config directories.

```sh
mkdir -p .dev/keys .dev/periphery
```

Add `.dev/core.config.toml` with the following contents:

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

Create `ui/.env.development` with the following:

```
VITE_KOMODO_HOST=http://localhost:9120
```

Make sure your Rust toolchain is up to date and install the CLI tools:

```sh
rustup update
cargo install typeshare-cli runnables-cli
run yarn-install
```

### Starting the services

Start a Mongo instance in Docker:

```sh
docker run -d --name komodo-mongo \
-p 27017:27017 \
-v komodo-mongo-data:/data/db \
-v komodo-mongo-config:/data/configdb \
-e MONGO_INITDB_ROOT_USERNAME=komodo \
-e MONGO_INITDB_ROOT_PASSWORD=komodo \
mongo
```

In separate terminals, run Core, Periphery, and UI.

```sh
run dev-core
```

```sh
run dev-periphery
```

```sh
run dev-ui      # Start in dev (watch) mode
```

Once everything is running, open `http://localhost:5173` and create a user account.

### Rebuilding the frontend client

After API changes, rebuild the client with

```bash
run gen-client  # Rebuild client (after API changes)
```

## Docsite Development

Use `run dev-docsite` to start the [Docusaurus](https://docusaurus.io/) Komodo docs site in development mode. Changes made to files in `./docsite` will be automatically reloaded by the server.
