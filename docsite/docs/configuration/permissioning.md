---
sidebar_label: Permissions
---

# Permissioning

Komodo has a layered permission model for non-admin users and groups. The goal is to grant access
to the intended resources and capabilities without exposing everything in Core.

## User Groups

Komodo can assign permissions directly to a user. User groups help when several users need the same
access model.

Users can belong to multiple groups and inherit the group's permissions. There is also an
`Everyone` mode for groups. When enabled, every user implicitly receives that group's permissions.

For permissioning at scale, define user groups in
[Resource Sync](../automate/sync-resources.md) instead of managing everything only in the UI.

## Permission Levels

There are four base permission levels on a resource:

1. **None**. The user has no access. The resource does not appear in the UI or API results. This
   is the default for non-admins unless `KOMODO_TRANSPARENT_MODE=true`.
2. **Read**. The user can see the resource and read its configuration. Configuration updates and
   executions are blocked.
3. **Execute**. The user can trigger actions on the resource, such as a build or redeploy, but
   cannot update its configuration.
4. **Write**. The user can execute actions, update configuration, and delete the resource.

Using `KOMODO_TRANSPARENT_MODE=true` makes `Read` the base level on all resources for all users.

## Specific Permissions

Some capabilities are gated separately from the base level.

- **`Logs`**: retrieve Docker or Docker Compose logs on the associated resource.
  - Valid on `Server`, `Stack`, `Deployment`.
  - Use this when users need operational visibility without full execution or write access.
- **`Inspect`**: inspect Docker containers.
  - Valid on `Server`, `Stack`, `Deployment`.
  - **On Servers**: this exposes container environments on that server and can leak secrets if
    used too broadly.
- **`Terminal`**: access the associated resource's terminal.
  - On a `Server`, this grants server-level terminal access and container-exec privileges,
    including attached `Stacks` and `Deployments`.
  - On a `Stack` or `Deployment`, this grants container-exec terminal access even without
    `Terminal` on the `Server`.
- **`Attach`**: attach other resources to the resource.
  - On a `Server`, users can attach `Stacks`, `Deployments`, `Repos`, and `Builders`.
  - On a `Builder`, users can attach `Builds` and `Repos`.
  - On a `Build`, users can attach it to `Deployments`.
  - On a `Repo`, users can attach it to `Stacks`, `Builds`, and `Resource Syncs`.
- **`Processes`**: retrieve the full running process list on a `Server`.

## Permissioning By Resource Type

Users or groups can also receive a base permission level on all resources of one type, such as all
Stacks.

In TOML form:

```toml
[[user_group]]
name = "groupo"
users = ["mbecker20", "karamvirsingh98"]
all.Build = "Execute"
all.Stack = { level = "Read", specific = ["Logs"] }
```

A user or group can still receive a stronger permission level on selected resources:

```toml
permissions = [
  # Grant additional specific permissions.
  {
    target.type = "Stack",
    target.id = "my-stack",
    level = "Execute",
    specific = ["Inspect", "Terminal"],
  },
  # Use regex to match multiple resources.
  { target.type = "Stack", target.id = "\\^john-(.+)$\\", level = "Execute" },
]
```

## Administration

Users can be given admin privileges by a `Super Admin`. Only the first user has this status by
default. A super admin can grant or remove admin status for other users.

Admins have unrestricted access to Komodo resources. They can also update other non-admin users'
permissions.

Admins also manage user accounts. When a user logs in for the first time, they are not immediately
granted access unless `KOMODO_ENABLE_NEW_USERS=true`. An admin must first enable the user from
`Settings > Users`. Admins can also disable users at any time, which blocks their GUI and API
access.

Users also have some configurable global permissions:

- create server permission
- create build permission

Only users with these permissions, along with admins, can add servers and create builds.
If `disable_non_admin_create = true`, only admins can create resources regardless of those flags.

## Related Pages

- [Server](../server.md)
- [Resources](../resources.md)
- [Terminals](../terminals.md)
- [Resource Sync](../automate/sync-resources.md)
