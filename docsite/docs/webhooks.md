# Configuring Webhooks

Multiple Komodo resources can take advantage of webhooks from your git provider. Komodo supports incoming webhooks using either the Github or Gitlab webhook authentication type, which is also supported by other providers like Gitea.

:::note
On Gitea, the default "Gitea" webhook type works with the Github authentication type 👍
:::

## Copy the Webhook URL

Find the resource in UI, like a `Build`, `Repo`, or `Stack`.
Go to the `Config` section, find "Webhooks", and copy the webhook for the action you want.

The webhook URL is constructed as follows:

```shell
https://${HOST}/listener/${AUTH_TYPE}/${RESOURCE_TYPE}/${ID_OR_NAME}/${EXECUTION}
```
- **`HOST`**: Your Komodo endpoint to recieve webhooks. 
	- If your Komodo sits in a private network,
	  you will need a public proxy setup to forward `/listener` requests to Komodo.
- **`AUTH_TYPE`**:
	- options: `github` | `gitlab`
	- `github`: Validates the signature attached with `X-Hub-Signature-256`. [reference](https://docs.github.com/en/webhooks/using-webhooks/validating-webhook-deliveries)
	- `gitlab`: Checks that the secret attached to `X-Gitlab-Token` is valid. [reference](https://docs.gitlab.com/ee/user/project/integrations/webhooks.html#create-a-webhook)
- **`RESOURCE_TYPE`**:
	- options: `build` | `repo` | `stack` | `sync` | `procedure` | `action`
- **`ID_OR_NAME`**:
	- Reference the specific resource by id or name. If the name may change, it is better to use id.
- **`EXECUTION`**:
	- Which executions are available depends on the `RESOURCE_TYPE`. Builds only have the `/build` action.
		Repos can select between `/pull`, `/clone`, or `/build`. Stacks have `/deploy` and `/refresh`, and Resource Syncs have `/sync` and `/refresh`.
	- For **Procedures and Actions**, this will be the **branch to listen to for pushes**, or `__ANY__` to trigger
		on pushes to any branch.

## Create the webhook on the Git Provider

Navigate to the repo page on your git provider, and go to the settings for the Repo.
Find Webhook settings, and click to create a new webhook.

You will have to input some information. 

1. The `Payload URL` is the link that you copied in the step above, `Copy the Resource Payload URL`.
2. For Content-type, choose `application/json`
3. For Secret, input the secret you configured in the Komodo Core config (`KOMODO_WEBHOOK_SECRET`).
4. Enable SSL Verification, if you have proper TLS setup to your git provider (recommended).
5. For "events that trigger the webhook", just the push request is what most people want.
6. Of course, make sure the webhook is "Active" and hit create.

## When does it trigger?

Your git provider will now push this webhook to Komodo on *every* push to *any* branch. However, your `Build`, `Repo`,
etc. only cares about a specific branch of the repo.

Because of this, the webhook will trigger the action **only on pushes to the branch configured on the resource**.

For example, if I make a build, I may point the build to the `release` branch of a particular repo. If I set up a webhook, and push to the `main` branch, the action will *not trigger*. It will only trigger when the push is to the `release` branch.
