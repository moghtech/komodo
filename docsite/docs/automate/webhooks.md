# Webhooks

Komodo resources can be triggered by incoming webhooks from your git provider. GitHub and GitLab authentication types are supported, which also covers Gitea, Forgejo, and other compatible providers.

:::note
Gitea's default "Gitea" webhook type works with the GitHub authentication type.
:::

## Webhook URL

Find the webhook URL on any resource's Config page under "Webhooks". The URL format is:

```
https://<HOST>/listener/<AUTH_TYPE>/<RESOURCE_TYPE>/<ID_OR_NAME>/<EXECUTION>
```

| Component | Options |
|---|---|
| `HOST` | Your Komodo endpoint. If Komodo is on a private network, set up a public proxy for `/listener` requests. |
| `AUTH_TYPE` | `github` — validates `X-Hub-Signature-256`. `gitlab` — validates `X-Gitlab-Token`. |
| `RESOURCE_TYPE` | `build`, `repo`, `stack`, `sync`, `procedure`, `action` |
| `ID_OR_NAME` | Resource ID or name. Use ID if the name may change. |
| `EXECUTION` | Depends on resource type (see below). |

### Executions by resource type

| Resource | Available executions |
|---|---|
| Build | `/build` |
| Repo | `/pull`, `/clone`, `/build` |
| Stack | `/deploy`, `/refresh` |
| Resource Sync | `/sync`, `/refresh` |
| Procedure / Action | Branch name to listen for (e.g. `/main`), or `/__ANY__` for all branches. |

## Setting Up a Webhook

1. Copy the webhook URL from the resource's Config page in Komodo.
2. On your git provider, go to the repo's **Settings > Webhooks** and create a new webhook.
3. Set the **Payload URL** to the copied URL.
4. Set **Content-type** to `application/json`.
5. Set **Secret** to your `KOMODO_WEBHOOK_SECRET`.
6. Select **Push events** as the trigger.

## Branch Filtering

Your git provider sends webhooks on pushes to **any** branch. Komodo only triggers the action when the push matches the **branch configured on the resource**. For example, a Build pointed at the `release` branch will ignore pushes to `main`.
