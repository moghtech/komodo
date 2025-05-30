# Sync Resources

Komodo is able to create, update, delete, and deploy resources declared in TOML files by diffing them against the existing resources,
and apply updates based on the diffs. Similar to Stacks, the files can be configured in UI, in a local file, or in files pushed to a remote git repo.
The Komodo Core backend will poll the files for for any updates, and alert about pending changes when diffs are detected.

You can spread out your resource declarations across any number of files
and use any nesting of folders to organize resources inside a root folder.
Additionally, you can create multiple `ResourceSyncs` and configure `Match Tags` to filter down which resources are synced,
and each sync will be handled independently. This allows different syncs to manage resources on a "per-project" basis.

The UI will display the computed sync actions and only execute them upon manual confirmation.
Or the sync execution git webhook may be configured on the git repo to
automatically execute syncs upon pushes to the configured branch.

## Commit to Syncs

If the Sync is pointing to just a single file, you can enable "Managed Mode" to allow Core to write the updates you made in UI _back to the file_.
This works no matter where the files are located, and will create a commit to your git repository for repo based files.

## Example Declarations

### Server

- [Server config schema](https://docs.rs/komodo_client/latest/komodo_client/entities/server/struct.ServerConfig.html)

```toml
[[server]] # Declare a new server
name = "server-prod"
description = "the prod server"
tags = ["prod"]
[server.config]
address = "http://localhost:8120"
region = "AshburnDc1"
enabled = true # default: false
```

### Builder and build

- [Builder config schema](https://docs.rs/komodo_client/latest/komodo_client/entities/builder/enum.BuilderConfig.html)
- [Build config schema](https://docs.rs/komodo_client/latest/komodo_client/entities/build/struct.BuildConfig.html)

```toml
[[builder]] # Declare a builder
name = "builder-01"
tags = []
config.type = "Aws"
[builder.config.params]
region = "us-east-2"
ami_id = "ami-0e9bd154667944680"
# These things come from your specific setup
subnet_id = "subnet-xxxxxxxxxxxxxxxxxx"
key_pair_name = "xxxxxxxx"
assign_public_ip = true
use_public_ip = true
security_group_ids = [
  "sg-xxxxxxxxxxxxxxxxxx",
  "sg-xxxxxxxxxxxxxxxxxx"
]

##

[[build]]
name = "test_logger"
description = "Logs randomly at INFO, WARN, ERROR levels to test logging setups"
tags = ["test"]
[build.config]
builder_id = "builder-01"
repo = "mbecker20/test_logger"
branch = "master"
git_account = "mbecker20"
image_registry.type = "Standard"
image_registry.params.domain = "github.com" # or your custom domain
image_registry.params.account = "your_username"
image_registry.params.organization = "your_organization" # optinoal
# Set docker labels
labels = """
org.opencontainers.image.source = https://github.com/mbecker20/test_logger
org.opencontainers.image.description = Logs randomly at INFO, WARN, ERROR levels to test logging setups
org.opencontainers.image.licenses = GPL-3.0
"""
```

### Deployments

- [Deployment config schema](https://docs.rs/komodo_client/latest/komodo_client/entities/deployment/struct.DeploymentConfig.html)

```toml
# Declare variables
[[variable]]
name = "OTLP_ENDPOINT"
value = "http://localhost:4317"

##

[[deployment]] # Declare a deployment
name = "test-logger-01"
description = "test logger deployment 1"
tags = ["test"]
# sync will deploy the container:
#  - if it is not running.
#  - has relevant config updates.
#  - the attached build has new version.
deploy = true
[deployment.config]
server_id = "server-01"
image.type = "Build"
image.params.build = "test_logger"
# set the volumes / bind mounts
volumes = """
# Supports comments
/data/logs = /etc/logs
# And other formats (eg yaml list)
- "/data/config:/etc/config"
"""
# Set the environment variables
environment = """
# Comments supported
OTLP_ENDPOINT = [[OTLP_ENDPOINT]] # interpolate variables into the envs.
VARIABLE_1 = value_1
VARIABLE_2 = value_2
"""
# Set Docker labels
labels = "deployment.type = logger"

##

[[deployment]]
name = "test-logger-02"
description = "test logger deployment 2"
tags = ["test"]
deploy = true
# Create a dependency on test-logger-01. This deployment will only be deployed after test-logger-01 is deployed.
# Additionally, any sync deploy of test-logger-01 will also trigger sync deploy of this deployment.
after = ["test-logger-01"]
[deployment.config]
server_id = "server-01"
image.type = "Build"
image.params.build = "test_logger"
volumes = """
/data/logs = /etc/logs
/data/config = /etc/config"""
environment = """
VARIABLE_1 = value_1
VARIABLE_2 = value_2
"""
# Set Docker labels
labels = "deployment.type = logger"
```

### Stack

- [Stack config schema](https://docs.rs/komodo_client/latest/komodo_client/entities/stack/struct.StackConfig.html)

```toml
[[stack]]
name = "test-stack"
description = "stack test"
deploy = true
after = ["test-logger-01"] # Stacks can depend on deployments, and vice versa.
tags = ["test"]
[stack.config]
server_id = "server-prod"
file_paths = ["mongo.yaml", "redis.yaml"]
git_provider = "git.mogh.tech"
git_account = "mbecker20" # clone private repo by specifying account
repo = "mbecker20/stack_test"
```

### Procedure

- [Procedure config schema](https://docs.rs/komodo_client/latest/komodo_client/entities/procedure/struct.ProcedureConfig.html)

```toml
[[procedure]]
name = "test-procedure"
description = "Do some things in a specific order"
tags = ["test"]

[[procedure.config.stage]]
name = "Build stuff"
executions = [
  { execution.type = "RunBuild", execution.params.build = "test_logger" },
  # Uses the Batch version, witch matches many builds by pattern
  # This one matches all builds prefixed with `foo-` (wildcard) and `bar-` (regex).
  { execution.type = "BatchRunBuild", execution.params.pattern = "foo-* , \\^bar-.*$\\" },
  { execution.type = "PullRepo", execution.params.repo = "komodo-periphery" },
]

[[procedure.config.stage]]
name = "Deploy test logger 1"
executions = [
  { execution.type = "Deploy", execution.params.deployment = "test-logger-01" },
  { execution.type = "Deploy", execution.params.deployment = "test-logger-03", enabled = false },
]

[[procedure.config.stage]]
name = "Deploy test logger 2"
enabled = false
executions = [
  { execution.type = "Deploy", execution.params.deployment = "test-logger-02" }
]
```

### Repo

- [Repo config schema](https://docs.rs/komodo_client/latest/komodo_client/entities/repo/struct.RepoConfig.html)

```toml
[[repo]]
name = "komodo-periphery"
description = "Builds new versions of the periphery binary. Requires Rust installed on the host."
tags = ["komodo"]
[repo.config]
server_id = "server-01"
git_provider = "git.mogh.tech" # use an alternate git provider (default is github.com)
git_account = "mbecker20"
repo = "moghtech/komodo"
# Run an action after the repo is pulled
on_pull.path = "."
on_pull.command = """
# Supports comments
/root/.cargo/bin/cargo build -p komodo_periphery --release
# Multiple lines will be combined together using '&&'
cp ./target/release/periphery /root/periphery
"""
```

### Resource sync

- [Resource sync config schema](https://docs.rs/komodo_client/latest/komodo_client/entities/sync/type.ResourceSync.html)

```toml
[[resource_sync]]
name = "resource-sync"
[resource_sync.config]
git_provider = "git.mogh.tech" # use an alternate git provider (default is github.com)
git_account = "mbecker20"
repo = "moghtech/komodo"
resource_path = ["stacks.toml", "repos.toml"]
```

### User Group:

- [UserGroup schema](https://docs.rs/komodo_client/latest/komodo_client/entities/toml/struct.UserGroupToml.html)

```toml
[[user_group]]
name = "groupo"
everyone = false # Set to true to give these permission to all users.
users = ["mbecker20", "karamvirsingh98"]
# Configure write access with all specific permissions
all.Server = { level = "Write", specific = ["Attach", "Logs", "Inspect", "Terminal", "Processes"] }
# Attach base level of Execute on all builds
all.Build = "Execute"
# Allow users to see all Builders, and attach builds to them.
all.Builder = { level = "Read", specific = ["Attach"] }
permissions = [
  # Attach permissions to specific resources by name
  { target.type = "Repo", target.id = "komodo-periphery", level = "Execute" },
  # Attach permissions to many resources with name matching regex (this uses '^(.+)-(.+)$' as regex expression)
  { target.type = "Server", target.id = "\\^(.+)-(.+)$\\", level = "Read" },
  { target.type = "Deployment", target.id = "\\^immich\\", level = "Execute" },
]
```
