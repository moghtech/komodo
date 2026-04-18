# Variables and Secrets

A variable or secret in Komodo is a key-value pair.

```toml
KEY_1 = "value_1"
```

You can interpolate the value into resource environment fields and many other user-configurable
inputs, such as Repo `On Clone` and `On Pull` commands or Stack `Extra Args`, by wrapping the key in
double brackets:

```toml
# Before interpolation
SOME_ENV_VAR = [[KEY_1]]

# After interpolation
SOME_ENV_VAR = value_1
```

## Defining Variables And Secrets

Komodo supports four common ways to define these values:

- **In the UI**: use `Settings > Variables` to create values stored in the Komodo database.
  - Marking a variable as secret prevents its value from appearing in updates, logs, or non-admin
    API responses.
  - Variables can also be managed in [Resource Sync](../automate/sync-resources.md), but secret
    values should stay out of version-controlled files.
- **Mounted config file in Core**: see [Mounted Config Files](/docs/how-to/mounted-config-files).
  - In `core.config.toml`, define a `secrets` block:

    ```toml
    [secrets]
    KEY_1 = "value_1"
    KEY_2 = "value_2"
    ```

  - These keys are available for interpolation from Core-managed configuration, as if they were
    variables created in the UI.
  - The keys appear in the variables page, but the values are never exposed through the API.
- **Mounted config file in Periphery**:
  - `periphery.config.toml` supports the same `secrets` block syntax.
  - These values are only available to resources on the associated Server.
  - This keeps the secret local to the Periphery host and avoids sending it from Core to Periphery.
- **Dedicated secret management tool**:
  - For stricter enterprise requirements, use a system such as HashiCorp Vault alongside Komodo.
  - In that model, applications fetch secret values directly from the external secret manager rather
    than storing them in Komodo.
