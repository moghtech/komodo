# Development

If you are looking to contribute to Komodo, this page is a launching point for setting up your Komodo development environment.

## Dependencies

Running Komodo from [source](https://github.com/moghtech/komodo) requires either [Docker](https://www.docker.com/) (and can use the included [devcontainer](https://code.visualstudio.com/docs/devcontainers/containers)), or can have the development dependencies installed locally:

* Backend (Core / Periphery APIs)
    * [Rust](https://www.rust-lang.org/) stable via [rustup installer](https://rustup.rs/)
    * [MongoDB](https://www.mongodb.com/) or [FerretDB](https://www.ferretdb.com/) available locally.
    * On Debian/Ubuntu: `apt install build-essential pkg-config libssl-dev` required to build the rust source.
* Frontend (Web UI)
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

After opening the repository with the devcontainer run the task `Init` to build the frontend/backend. Then, the task `Run Komodo` can be used to run frontend/backend. Other tasks for rebuilding/running just one component of the stack (Core API, Periphery API, Frontend) are also provided.

## Local

To run a full Komodo instance from a non-container environment run commands in this order:

* Ensure dependencies are up to date
    * `rustup update` -- ensure rust toolchain is up to date
* Build and Run backend
    * `run dev-core` -- Build and run Core API
    * `run dev-periphery` -- Build and run Periphery API
* Build Frontend
    * Install **typeshare-cli**: `cargo install typeshare-cli`
    * **Run this once** -- `run link-client` -- generates TS client and links to the frontend
    * After running the above once:
        * `run gen-client` -- Rebuild client 
        * `run dev-frontend` -- Start in dev (watch) mode
        * `run build-frontend` -- Typecheck and build
            

## Docsite Development

Use `run dev-docsite` to start the [Docusaurus](https://docusaurus.io/) Komodo docs site in development mode. Changes made to files in `./docsite` will be automatically reloaded by the server.