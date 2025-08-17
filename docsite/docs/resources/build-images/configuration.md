# Configuration

Komodo just needs a bit of information in order to build your image.

### Provider configuration
Komodo supports cloning repos over http/s, from any provider that supports cloning private repos using `git clone https://<Token>@git-provider.net/<Owner>/<Repo>`.

Accounts / access tokens can be configured in either the [core config](../../setup/advanced.mdx#mount-a-config-file)
or in the [periphery config](../../setup/connect-servers.mdx#manual-install-steps---binaries).

### Repo configuration
To specify the git repo to build, just give it the name of the repo and the branch under *repo config*. The name is given like `moghtech/komodo`, it includes the username / organization that owns the repo.

Many repos are private, in this case an access token is needed by the building server.
It can either come from a provider defined in the core configuration,
or in the periphery configuration of the building server.

### Docker build configuration

In order to docker build, Komodo just needs to know the build directory and the path of the Dockerfile relative to the repo, you can configure these in the *build config* section.

If the build directory is the root of the repository, you pass the build path as `.`. If the build directory is some folder of the repo, just pass the name of the the folder. Do not pass the preceding "/". for example `build/directory`

The dockerfile's path is given relative to the build directory. So if your build directory is `build/directory` and the dockerfile is in `build/directory/Dockerfile.example`, you give the dockerfile path simply as `Dockerfile.example`.

### Image registry

Komodo supports pushing to any docker registry. 
Any of the accounts that are specified in config for the specific registry, between the core config and builder, will be available to use for authentication against the registry.
Additionally, allowed organizations on the docker registry can be specified on the core config and attached to builds.
Doing so will cause the images to be published under the organization's namespace rather than the account's.

When connecting a build to a deployments, the default behavior is for the deployment to inherit the registry configuration from the build.
In cases where that account isn't available to the deployment, another account can be chosen in the deployment config.

:::note
In order to publish to the Github Container Registry, your Github access token must be given the `write:packages` permission.
See the Github docs [here](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#authenticating-with-a-personal-access-token-classic).
:::

### Adding build args

The Dockerfile may make use of [build args](https://docs.docker.com/engine/reference/builder/#arg). Build args can be passed using the gui by navigating to the `Build Args` tab in the config. They are passed in the menu just like in the would in a .env file:

```
BUILD_ARG1=some_value
BUILD_ARG2=some_other_value
```

Note that these values are visible in the final image using `docker history`, so shouldn't be used to pass build time secrets. Use [secret mounts](https://docs.docker.com/engine/reference/builder/#run---mounttypesecret) for this instead.

### Adding build secrets

The Dockerfile may also make use of [build secrets](https://docs.docker.com/build/building/secrets).

They are configured in the GUI the same way as build args. The values passed here can be used in RUN commands in the Dockerfile:
```
RUN --mount=type=secret,id=SECRET_KEY \
  SECRET_KEY=$(cat /run/secrets/SECRET_KEY) ...
```

These values will not be visible with `docker history` command.