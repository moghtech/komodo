import {
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
import { Group, Stack, Text } from "@mantine/core";
import { ProviderSelectorConfig } from "@/components/config/provider-selector";
import { AccountSelectorConfig } from "@/components/config/account-selector";
import SecretsSearch from "@/components/config/secrets-search";
import { MonacoEditor } from "@/components/monaco";
import SystemCommand from "@/components/config/system-command";
import { ReactNode } from "react";
import CopyWebhookUrl from "@/components/webhook/copy-url";
import WebhookBuilder from "@/components/webhook/builder";

export default function RepoConfig({
  id,
  titleOther,
}: {
  id: string;
  titleOther?: ReactNode;
}) {
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
  const { getIntegration } = useWebhookIntegrations();
  const [idOrName] = useWebhookIdOrName();

  if (!config) return null;

  const disabled = global_disabled || !canWrite;

  const gitProvider = update.git_provider ?? config.git_provider;
  const webhookIntegration = getIntegration(gitProvider);

  return (
    <Config
      titleOther={titleOther}
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
                        <Group fz="h3">
                          Server:
                          <ResourceLink
                            type="Server"
                            id={server_id}
                            fz="h3"
                            iconSize="1.2rem"
                          />
                        </Group>
                      ) : (
                        "Select Server"
                      )
                    }
                    description="Select the Server to clone on. Optional."
                  >
                    <ResourceSelector
                      type="Server"
                      selected={server_id}
                      onSelect={(server_id) => set({ server_id })}
                      disabled={disabled}
                      clearable
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
                        <Group fz="h3">
                          Builder:
                          <ResourceLink
                            type="Builder"
                            id={builder_id}
                            fz="h3"
                            iconSize="1.2rem"
                          />
                        </Group>
                      ) : (
                        "Select Builder"
                      )
                    }
                    description="Select the Builder to build with. Optional."
                  >
                    <ResourceSelector
                      type="Builder"
                      selected={builder_id}
                      onSelect={(builder_id) => set({ builder_id })}
                      disabled={disabled}
                      clearable
                    />
                  </ConfigItem>
                );
              },
            },
          },
          {
            label: "Source",
            fields: {
              git_provider: (provider, set) => {
                const https = update.git_https ?? config.git_https;
                return (
                  <ProviderSelectorConfig
                    accountType="git"
                    selected={provider}
                    disabled={disabled}
                    onSelect={(git_provider) => set({ git_provider })}
                    https={https}
                    onHttpsSwitch={() => set({ git_https: !https })}
                  />
                );
              },
              git_account: (account, set) => (
                <AccountSelectorConfig
                  id={update.builder_id ?? config.builder_id ?? undefined}
                  accountType="git"
                  type="Builder"
                  provider={update.git_provider ?? config.git_provider}
                  selected={account}
                  onSelect={(git_account) => set({ git_account })}
                  disabled={disabled}
                  placeholder="None"
                />
              ),
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
                  <Stack gap="0">
                    <Text>
                      Explicitly specify the folder on the host to clone the
                      repo in.
                    </Text>
                    <Text>
                      If <b>relative path</b> (no leading '/'), relative to{" "}
                      <b>{"$root_directory/repos/" + repo.name}</b>
                    </Text>
                  </Stack>
                ),
              },
            },
          },
          {
            label: "Environment",
            description:
              "Write these variables to a .env-formatted file at the specified path before on_clone / on_pull are run.",
            fields: {
              environment: (env, set) => (
                <Stack gap="xs">
                  <SecretsSearch
                    server={update.server_id ?? config.server_id}
                  />
                  <MonacoEditor
                    value={env || "  # VARIABLE = value\n"}
                    onValueChange={(environment) => set({ environment })}
                    language="key_value"
                    readOnly={disabled}
                  />
                </Stack>
              ),
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
              on_clone: (value, set) => (
                <SystemCommand
                  value={value}
                  set={(value) => set({ on_clone: value })}
                  disabled={disabled}
                />
              ),
            },
          },
          {
            label: "On Pull",
            description:
              "Execute a shell command after pulling the repo. The given Cwd is relative to repo root.",
            fields: {
              on_pull: (value, set) => (
                <SystemCommand
                  value={value}
                  set={(value) => set({ on_pull: value })}
                  disabled={disabled}
                />
              ),
            },
          },
          {
            label: "Webhooks",
            description: `Copy the webhook given here, and  your ${webhookIntegration}-style repo provider to send webhooks to Komodo`,
            fields: {
              ["Guard" as any]: () => {
                if (update.branch ?? config.branch) {
                  return null;
                }
                return (
                  <ConfigItem label="Configure Branch">
                    <Text>
                      Must configure Branch before webhooks will work.
                    </Text>
                  </ConfigItem>
                );
              },
              ["Builder" as any]: () => (
                <WebhookBuilder gitProvider={gitProvider} />
              ),
              ["Pull" as any]: () => (
                <CopyWebhookUrl
                  label="Webhook Url - Pull"
                  integration={webhookIntegration}
                  path={`/repo/${idOrName === "Id" ? id : encodeURIComponent(name ?? "...")}/pull`}
                />
              ),
              ["Clone" as any]: () => (
                <CopyWebhookUrl
                  label="Webhook Url - Clone"
                  integration={webhookIntegration}
                  path={`/repo/${idOrName === "Id" ? id : encodeURIComponent(name ?? "...")}/clone`}
                />
              ),
              ["Build" as any]: () => (
                <CopyWebhookUrl
                  label="Webhook Url - Build"
                  integration={webhookIntegration}
                  path={`/repo/${idOrName === "Id" ? id : encodeURIComponent(name ?? "...")}/build`}
                />
              ),
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
