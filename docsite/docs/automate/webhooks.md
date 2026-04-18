# Webhooks

Komodo resources can be triggered by incoming webhooks from a git provider. GitHub and GitLab
authentication types are supported, which also covers Gitea, Forgejo, and other compatible
providers.

Use webhooks when a push to a repository should cause Komodo to build, pull, deploy, sync, or run
automation.

:::note
Gitea's default `Gitea` webhook type works with the GitHub authentication type.
:::

## Webhook URL

Find the webhook URL on a resource's Config page under `Webhooks`. The URL format is:

```text
https://<HOST>/listener/<AUTH_TYPE>/<RESOURCE_TYPE>/<ID_OR_NAME>/<EXECUTION>
```

| Component | Options |
| --- | --- |
| `HOST` | Your Komodo endpoint. If Komodo is private, expose `/listener` on a reachable path. |
| `AUTH_TYPE` | `github` validates `X-Hub-Signature-256`. `gitlab` validates `X-Gitlab-Token`. |
| `RESOURCE_TYPE` | `build`, `repo`, `stack`, `sync`, `procedure`, `action` |
| `ID_OR_NAME` | Resource ID or name. Use ID if the name may change. |
| `EXECUTION` | Depends on resource type. |

### Executions By Resource Type

| Resource | Available executions |
| --- | --- |
| Build | `/build` |
| Repo | `/pull`, `/clone`, `/build` |
| Stack | `/deploy`, `/refresh` |
| Resource Sync | `/sync`, `/refresh` |
| Procedure / Action | Branch name to listen for, such as `/main`, or `/__ANY__` for all branches |

## Set Up A Webhook

1. Copy the webhook URL from the resource's Config page in Komodo.
2. In your git provider, open the repository's **Settings > Webhooks** and create a new webhook.
3. Set the **Payload URL** to the copied URL.
4. Set **Content-type** to `application/json`.
5. Set **Secret** to your `KOMODO_WEBHOOK_SECRET`.
6. Select **Push events** as the trigger.

## Branch Filtering

Your git provider sends webhooks on pushes to any branch. Komodo only triggers the action when the
push matches the branch configured on the resource. For example, a Build pointed at the `release`
branch ignores pushes to `main`.

## Related Pages

- [Use A Reverse Proxy With Core And Periphery](../how-to/reverse-proxy-and-periphery-access.md)
- [Configure Git-Backed Workflows](../how-to/git-backed-workflows.md)
- [Repo](../repo.md)
- [Resource Sync](./sync-resources.md)
- [Procedures and Actions](./procedures.md)
