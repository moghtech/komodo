# Resources

Komodo is extendible through the **Resource** abstraction. Entities like `Server`, `Deployment`, and `Stack` are all **Komodo Resources**.

All resources have common traits, such as a unique `name` and `id` amongst all other resources of the same resource type.
All resources can be assigned `tags`, which can be used to group related resources.

:::note
Many resources need access to git repos / docker registries. There is an in-built token management system (managed in UI or in config file) to give resources access to credentials.
All resources which depend on git repos / docker registries are able to use these credentials to access private repos.
:::

## [Server](setup/connect-servers)

- Configure the connection to periphery agents.
- Set alerting thresholds.
- Can be attached to by **Deployments**, **Stacks**, **Repos**, and **Builders**.

## [Deployment](resources/deploy-containers/index.mdx)

- Deploy a docker container on the attached Server.
- Manage services at the container level, perform orchestration using **Procedures** and **ResourceSyncs**.

## [Stack](resources/docker-compose)

- Deploy with docker compose.
- Provide the compose file in UI, or move the files to a git repo and use a webhook for auto redeploy on push.
- Supports composing multiple compose files using `docker compose -f ... -f ...`.
- Pass environment variables usable within the compose file. Interpolate in app-wide variables / secrets.

## Repo

- Put scripts in git repos, and run them on a Server, or using a Builder.
- Can build binaries, perform automation, really whatever you can think of.

## [Build](resources/build-images/index.mdx)

- Build application source into docker images, and push them to the configured registry.
- The source can be any git repo containing a Dockerfile.

## [Builder](resources/build-images/builders)

- Either points to a connected server, or holds configuration to launch a single-use AWS instance to build the image.
- Can be attached to **Builds** and **Repos**.

## [Procedure](resources/procedures#procedures)

- Compose many actions on other resource type, like `RunBuild` or `DeployStack`, and run it on button push (or with a webhook).
- Can run one or more actions in parallel "stages", and compose a series of parallel stages to run sequentially.

## [Action](resources/procedures#actions)

- Write scripts calling the Komodo API in Typescript
- Use a pre-initialized Komodo client within the script, no api keys necessary.
- Type aware in UI editor. Get suggestions and see in depth docs as you type.
- The Typescript client is also [published on NPM](https://www.npmjs.com/package/komodo_client).

## [Resource Sync](resources/sync-resources)

- Orchestrate all your configuration declaratively by defining it in `toml` files, which are checked into a git repo.
- Can deploy **Deployments** and **Stacks** if changes are suggested.
- Specify deploy ordering with `after` array. (like docker compose `depends_on` but can span across servers.).

## Alerter

- Route alerts to various endpoints.
- Can configure rules on each Alerter, such as resource whitelist, blacklist, or alert type filter.
