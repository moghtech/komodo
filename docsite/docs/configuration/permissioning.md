# Permissioning

Komodo has a granular, layer-based permissioning system to provide non-admin users access only to intended Resources.

## User Groups

While Komodo can assign permissions to specific users directly, it is recommended to instead **create User Groups and assign permissions to them**, as if they were a user.

Users can then be **added to multiple User Groups** and they **inherit the group's permissions**, similar to linux permissions.
There is also an `Everyone` mode for User Groups, if this is enabled then **all users implicitly gain the groups permissions**.

For permissioning at scale, users can define [**User Groups in Resource Syncs**](/docs/automate/sync-resources#user-group).

## Permission Levels

There are 4 permission levels a user / group can be given on a Resource:

 1. **None**. The user will not have any access to the resource. The user **will not see it in the GUI, and it will not show up if the user queries the Komodo API directly**. All attempts to view or update the resource will be blocked. This is the default for non-admins, unless using `KOMODO_TRANSPARENT_MODE=true`.

 2. **Read**. This is the first permission level that grants any access. It will enable the user to **see the resource in the GUI and read the configuration**. Any attempts to update configuration or trigger any action **will be blocked**.  Using `KOMODO_TRANSPARENT_MODE=true` will make this level the base level on all resources, for all users.

 3. **Execute**. This level will allow the user to execute actions on the resource, **like send a build command** or **trigger a redeploy**. The user will still be blocked from updating configuration on the resource.

 4. **Write**. The user has full config write access to the resource, **they can execute any actions, update the configuration, and delete the resource**.

## Specific Permissions

Permission levels alone are not quite enough to provide granular access control.
Some features are additionally gated behind a specific permission for that feature.

- **`Logs`**: User can retrieve docker / docker compose logs on the associated resource.
  - Valid on `Server`, `Stack`, `Deployment`.
  - For admins wanting this permission by default for all users with read permissions, see below on default user groups.
- **`Inspect`**:  User can "inspect" docker containers.
  - Valid on `Server`, `Stack`, `Deployment`.
  - **On Servers**: Access to this api will expose all container environments on the given server,
  and can easily lead to secrets being leaked to unintended users if not protected.
- **`Terminal`**: User can access the associated resource's terminal.
  - If given on a `Server`, this allows server level terminal access, and all container exec priviledges (Including attached `Stacks` / `Deployments`).
  - If given on a `Stack` or `Deployment`, this allows container exec terminal (even without `Terminal` on `Server`).
- **`Attach`**: User can "attach" *other resources* to the resource.
  - If given on a `Server`, allows users to attach `Stacks`, `Deployments`, `Repos`, and `Builders`.
  - If given on a `Builder`, allows users to attach `Builds` and `Repos`.
  - If given on a `Build`, allows users to attach it to `Deployments`.
  - If given on a `Repo`, allows users to attach it to `Stacks`, `Builds`, and `Resource Syncs`.
- **`Processes`**: User can retrieve the full running process list on the `Server`.

## Permissioning by Resource Type

Users or User Groups can be given a base permission level on all Resources of a particular type, such as Stack.
In TOML form, this looks like:

```toml
[[user_group]]
name = "groupo"
users = ["mbecker20", "karamvirsingh98"]
all.Build = "Execute" # <- Group members can run all builds (but not update config),
all.Stack = { level = "Read", specific = ["Logs"] }    # <- And see all Stacks / logs (no deploy / update, inspect, or terminal access).
```

A user / group can still be given a greater permission level on select resources:

```toml
permissions = [
  # Grant addition specific permission (Logs are already granted above)
  { target.type = "Stack", target.id = "my-stack", level = "Execute", specific = ["Inspect", "Terminal"] },
  # Use regex to match multiple resources, for example give john execute on all of their Stacks
  { target.type = "Stack", target.id = "\\^john-(.+)$\\", level = "Execute" },
]
```

## Administration

Users can be given Admin privileges by a `Super Admin` (only the first user is given this status, set with `super_admin: true` on a User document in database). Super admins will see the "Make Admin" button when on a User page `/users/${user_id}`.

These users have unrestricted access to all Komodo Resources. Additionally, these users can update other (non-admin) user's permissions on resources.

Komodo admins are responsible for managing user accounts as well. When a user logs into Komodo for the first time, they will not immediately be granted access (this can changed with `KOMODO_ENABLE_NEW_USERS=true`). An admin must first **enable** the user, which can be done from the `Users` tab on `Settings` page. Users can also be **disabled** by an admin at any time, which blocks all their access to the GUI and API. 

Users also have some configurable global permissions, these are:

 - create server permission
 - create build permission

Only users with these permissions (as well as admins) can add additional servers to Komodo, and can create additional builds, respectively.

## External Secret Provider Grants

External [Secret Providers](./secret-providers.md) (`Settings → Providers → Secrets`) enforce a **default-deny** authorization layer on top of standard Komodo resource permissions:

1. **Provider Management (`Admin` only)**: Only Komodo admins can create, update, test, disable, delete, or view full `SecretProvider` configurations and `SecretProviderGrant`s. No Read API ever returns stored credentials or encrypted ciphertexts, even to admins.
2. **Resource Usage (`SecretProviderGrant`)**: Even if a user (or admin) holds `Read`, `Execute`, or `Write` permission on a `Stack`, `Deployment`, or `Build`, listing remote secret keys, previewing secret bindings, or resolving secrets at deploy/build time requires a matching enabled `SecretProviderGrant` covering:
   - **Subject**: `User`, `UserGroup`, `ResourceAuthorized` (any enabled real user holding the required Komodo permission on the target resource), or `SystemActor`.
   - **Resource Scope**: Specific `Stack` / `Deployment` / `Build` IDs or `AllStacks` / `AllDeployments` / `AllBuilds`.
   - **Remote Selector Scope**: Allowed Infisical `project_ids`, `environments`, normalized `path_prefixes`, and explicit opt-in flags for `allow_imports` and `allow_expand_references`.
   - **Allowed Usages**: `MetadataList`, `Preview`, `StackEnv`, `StackComposeFileSecret`, `DeploymentEnv`, `BuildSecret`.
3. **Automated / Indirect Executions (`SystemActor`)**: Built-in internal system actors (`Procedure`, `Webhook`, `Schedule`, `AutoRedeploy`, `ResourceSync`, `Action`) **never** inherit implicit admin access to external secret providers. Unattended executions must be explicitly authorized via a `SystemActor` grant (optionally restricted to a specific `actor_resource_id` such as a single Procedure or ResourceSync ID).
