# Variables and Secrets

A variable / secret in Komodo is just a key-value pair.

```
KEY_1 = "value_1"
```

You can interpolate the value into any resource Environment
(and most other user configurable inputs, such as Repo `On Clone` and `On Pull`, or Stack `Extra Args`)
using double brackets around the key to trigger interpolation:

```toml
# Before interpolation
SOME_ENV_VAR = [[KEY_1]] # <- wrap the key in double brackets '[[]]'

# After iterpolation:
SOME_ENV_VAR = value_1
```

## Defining Variables and Secrets

- **In the UI**, you can go to `Settings` page, `Variables` tab. Here, you can create some Variables to store in the Komodo database.
  - There is a "secret" option you can check, this will **prevent the value from exposure in any updates / logs**, as well as prevent access to the value to any **non-admin** Komodo users.
  - Variables can also be managed in ResourceSyncs (see [example](/docs/resources/sync-resources#deployments)) but should only be done for non-secret variables, to avoid committing sensitive data. You should manage secrets using one of the following options.

- **Mount a config file to Core**: https://komo.do/docs/setup/advanced#mount-a-config-file
  - In the Komodo Core config file, you can configure `secrets` using a block like:
		```toml
		# in core.config.toml
		[secrets]
		KEY_1 = "value_1"
		KEY_2 = "value_2"
		```
  - `KEY_1` and `KEY_2` will be available for interpolation on all your resources, as if they were Variables set up in the UI.
  - They keys are queryable and show up on the variable page (so you know they are available for use),
		but **the values are not exposed by API for ANY user**.

- **Mount a config file to Periphery agent**:

  - In the Komodo Periphery config file, you can also configure `secrets` using the same syntax as the Core config file.
  - The variable **WILL NOT be available globally to all Komodo resources**, it will only be available to the resources on the associated Server resource on which that single Periphery agent is running.
  - This effectively distributes your secret locations, can be good or bad depending on your security requirements. It does avoid the need to send the secret over network from Core to Periphery, Periphery based secrets are never exposed to the network.

- **Use a dedicated secret management tool** such as Hashicorp Vault, alongside Komodo
  - Ultimately Komodo variable / secret features **may not fill enterprise level secret management requirements**, organizations of this level should use still a dedicated secret management solution. At this point Komodo is not intended as an enterprise level secret management solution.
  - These solutions do require application level integrations, your applications should only receive credentials to access the secret management API. **Your applications will pull the actual secret values from the dedicated secret management tool, they stay out of Komodo entirely**.
