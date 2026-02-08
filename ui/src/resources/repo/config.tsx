import {
  getWebhookIntegration,
  usePermissions,
  useRead,
  useWebhookIdOrName,
  useWebhookIntegrations,
  useWrite,
} from "@/lib/hooks";
import Config from "@/ui/config";
import { ConfigItem, ConfigList } from "@/ui/config/item";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import ResourceLink from "@/resources/link";
import ResourceSelector from "@/resources/selector";
import { Group } from "@mantine/core";

export default function RepoConfig({ id }: { id: string }) {
  const { canWrite } = usePermissions({ type: "Repo", id });
  const repo = useRead("GetRepo", { repo: id }).data;
  const config = repo?.config;
  const name = repo?.name;
  const global_disabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const [update, setUpdate] = useLocalStorage<Partial<Types.RepoConfig>>({
    key: `repo-${id}-update-v1`,
    defaultValue: {},
  });
  const { mutateAsync } = useWrite("UpdateRepo");
  const { integrations } = useWebhookIntegrations();
  const [id_or_name] = useWebhookIdOrName();

  if (!config) return null;

  const disabled = global_disabled || !canWrite;

  const git_provider = update.git_provider ?? config.git_provider;
  const webhook_integration = getWebhookIntegration(integrations, git_provider);

  return (
    <Config
      disabled={disabled}
      original={config}
      update={update}
      setUpdate={setUpdate}
      onSave={() => mutateAsync({ id, config: update })}
      groups={{
        "": [
          {
            label: "Server",
            labelHidden: true,
            fields: {
              server_id: (server_id, set) => {
                return (
                  <ConfigItem
                    label={
                      server_id ? (
                        <Group gap="xs">
                          Server:
                          <ResourceLink type="Server" id={server_id} />
                        </Group>
                      ) : (
                        "Select Server"
                      )
                    }
                    description="Select the Server to clone on."
                  >
                    <ResourceSelector
                      type="Server"
                      selected={server_id}
                      onSelect={(server_id) => set({ server_id })}
                      disabled={disabled}
                      position="bottom-start"
                    />
                  </ConfigItem>
                );
              },
            },
          },
          {
            label: "Builder",
            labelHidden: true,
            fields: {
              builder_id: (builder_id, set) => {
                return (
                  <ConfigItem
                    label={
                      builder_id ? (
                        <Group gap="xs">
                          Builder:
                          <ResourceLink type="Builder" id={builder_id} />
                        </Group>
                      ) : (
                        "Select Builder"
                      )
                    }
                    description="Select the Builder to build with."
                  >
                    <ResourceSelector
                      type="Builder"
                      selected={builder_id}
                      onSelect={(builder_id) => set({ builder_id })}
                      disabled={disabled}
                      position="bottom-start"
                    />
                  </ConfigItem>
                );
              },
            },
          },
          {
            label: "Source",
            fields: {
              // git_provider: (provider, set) => {
              //   const https = update.git_https ?? config.git_https;
              //   return (
              //     <ProviderSelectorConfig
              //       account_type="git"
              //       selected={provider}
              //       disabled={disabled}
              //       onSelect={(git_provider) => set({ git_provider })}
              //       https={https}
              //       onHttpsSwitch={() => set({ git_https: !https })}
              //     />
              //   );
              // },
              // git_account: (account, set) => (
              //   <AccountSelectorConfig
              //     id={update.builder_id ?? config.builder_id ?? undefined}
              //     type="Builder"
              //     account_type="git"
              //     provider={update.git_provider ?? config.git_provider}
              //     selected={account}
              //     onSelect={(git_account) => set({ git_account })}
              //     disabled={disabled}
              //     placeholder="None"
              //   />
              // ),
              repo: {
                placeholder: "Enter repo",
                description:
                  "The repo path on the provider. {namespace}/{repo_name}",
              },
              branch: {
                placeholder: "Enter branch",
                description: "Select a custom branch, or default to 'main'.",
              },
              commit: {
                label: "Commit Hash",
                placeholder: "Input commit hash",
                description:
                  "Optional. Switch to a specific commit hash after cloning the branch.",
              },
            },
          },
          {
            label: "Path",
            labelHidden: true,
            fields: {
              path: {
                label: "Clone Path",
                boldLabel: true,
                placeholder: "/clone/path/on/host",
                description: (
                  <div className="flex flex-col gap-0">
                    <div>
                      Explicitly specify the folder on the host to clone the
                      repo in.
                    </div>
                    <div>
                      If <span className="font-bold">relative</span> (no leading
                      '/'), relative to {"$root_directory/repos/" + repo.name}
                    </div>
                  </div>
                ),
              },
            },
          },
          {
            label: "Environment",
            description:
              "Write these variables to a .env-formatted file at the specified path, before on_clone / on_pull are run.",
            fields: {
              // environment: (env, set) => (
              //   <div className="flex flex-col gap-4">
              //     <SecretsSearch
              //       server={update.server_id ?? config.server_id}
              //     />
              //     <MonacoEditor
              //       value={env || "  # VARIABLE = value\n"}
              //       onValueChange={(environment) => set({ environment })}
              //       language="key_value"
              //       readOnly={disabled}
              //     />
              //   </div>
              // ),
              env_file_path: {
                description:
                  "The path to write the file to, relative to the root of the repo.",
                placeholder: ".env",
              },
              // skip_secret_interp: true,
            },
          },
          {
            label: "On Clone",
            description:
              "Execute a shell command after cloning the repo. The given Cwd is relative to repo root.",
            fields: {
              // on_clone: (value, set) => (
              //   <SystemCommand
              //     value={value}
              //     set={(value) => set({ on_clone: value })}
              //     disabled={disabled}
              //   />
              // ),
            },
          },
          {
            label: "On Pull",
            description:
              "Execute a shell command after pulling the repo. The given Cwd is relative to repo root.",
            fields: {
              // on_pull: (value, set) => (
              //   <SystemCommand
              //     value={value}
              //     set={(value) => set({ on_pull: value })}
              //     disabled={disabled}
              //   />
              // ),
            },
          },
          {
            label: "Webhooks",
            description: `Copy the webhook given here, and configure your ${webhook_integration}-style repo provider to send webhooks to Komodo`,
            fields: {
              // ["Guard" as any]: () => {
              //   if (update.branch ?? config.branch) {
              //     return null;
              //   }
              //   return (
              //     <ConfigItem label="Configure Branch">
              //       <div>Must configure Branch before webhooks will work.</div>
              //     </ConfigItem>
              //   );
              // },
              // ["Builder" as any]: () => (
              //   <WebhookBuilder git_provider={git_provider} />
              // ),
              // ["pull" as any]: () => (
              //   <ConfigItem label="Webhook Url - Pull">
              //     <CopyWebhook
              //       integration={webhook_integration}
              //       path={`/repo/${id_or_name === "Id" ? id : encodeURIComponent(name ?? "...")}/pull`}
              //     />
              //   </ConfigItem>
              // ),
              // ["clone" as any]: () => (
              //   <ConfigItem label="Webhook Url - Clone">
              //     <CopyWebhook
              //       integration={webhook_integration}
              //       path={`/repo/${id_or_name === "Id" ? id : encodeURIComponent(name ?? "...")}/clone`}
              //     />
              //   </ConfigItem>
              // ),
              // ["build" as any]: () => (
              //   <ConfigItem label="Webhook Url - Build">
              //     <CopyWebhook
              //       integration={webhook_integration}
              //       path={`/repo/${id_or_name === "Id" ? id : encodeURIComponent(name ?? "...")}/build`}
              //     />
              //   </ConfigItem>
              // ),
              webhook_enabled: true,
              webhook_secret: {
                description:
                  "Provide a custom webhook secret for this resource, or use the global default.",
                placeholder: "Input custom secret",
              },
            },
          },
          {
            label: "Links",
            description: "Add quick links in the resource header",
            contentHidden: ((update.links ?? config.links)?.length ?? 0) === 0,
            fields: {
              links: (values, set) => (
                <ConfigList
                  field="links"
                  values={values ?? []}
                  set={set}
                  disabled={disabled}
                  placeholder="Input link"
                />
              ),
            },
          },
        ],
      }}
    />
  );
}
