import { AccountSelectorConfig } from "@/components/config/account-selector";
import LinkedRepo from "@/components/config/linked-repo";
import { ProviderSelectorConfig } from "@/components/config/provider-selector";
import { MonacoEditor } from "@/components/monaco";
import Tags from "@/components/tags";
import TagSelector from "@/components/tags/selector";
import WebhookBuilder from "@/components/webhook/builder";
import CopyWebhookUrl from "@/components/webhook/copy-url";
import {
  usePermissions,
  useRead,
  useWebhookIdOrName,
  useWebhookIntegrations,
  useWrite,
} from "@/lib/hooks";
import Config, { ConfigGroupArgs, ConfigProps } from "@/ui/config";
import { ConfigItem, ConfigList, ConfigSwitch } from "@/ui/config/item";
import ShowHideButton from "@/ui/show-hide-button";
import { Group, Select, Text } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import { CircleMinus } from "lucide-react";
import { ReactNode } from "react";
import { useFullResourceSync } from ".";

type SyncMode = "UI Defined" | "Files On Server" | "Git Repo" | undefined;
const SYNC_MODES = ["UI Defined", "Files On Server", "Git Repo"] as const;

function getSyncMode(
  update: Partial<Types.ResourceSyncConfig>,
  config: Types.ResourceSyncConfig,
): SyncMode {
  if (update.files_on_host ?? config.files_on_host) return "Files On Server";
  if (
    (update.repo ?? config.repo) ||
    (update.linked_repo ?? config.linked_repo)
  )
    return "Git Repo";
  if (update.file_contents ?? config.file_contents) return "UI Defined";
  return undefined;
}

export default function ResourceSyncConfig({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const [show, setShow] = useLocalStorage({
    key: `sync-${id}-show`,
    defaultValue: {
      file: true,
      git: true,
      webhooks: true,
    },
  });
  const { canWrite } = usePermissions({ type: "ResourceSync", id });
  const sync = useFullResourceSync(id);
  const config = sync?.config;
  const name = sync?.name;
  const globalDisabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const [update, setUpdate] = useLocalStorage<
    Partial<Types.ResourceSyncConfig>
  >({
    key: `sync-${id}-update-v1`,
    defaultValue: {},
  });
  const { mutateAsync } = useWrite("UpdateResourceSync");
  const { getIntegration } = useWebhookIntegrations();
  const [idOrName] = useWebhookIdOrName();

  if (!config) return null;

  const disabled = globalDisabled || !canWrite;

  const gitProvider = update.git_provider ?? config.git_provider;
  const webhookIntegration = getIntegration(gitProvider);

  const mode = getSyncMode(update, config);
  const managed = update.managed ?? config.managed ?? false;

  const setMode = (mode: SyncMode) => {
    if (mode === "Files On Server") {
      setUpdate({ ...update, files_on_host: true });
    } else if (mode === "Git Repo") {
      setUpdate({
        ...update,
        files_on_host: false,
        repo: update.repo || config.repo || "namespace/repo",
      });
    } else if (mode === "UI Defined") {
      setUpdate({
        ...update,
        files_on_host: false,
        repo: "",
        file_contents:
          update.file_contents ||
          config.file_contents ||
          "# Initialize the sync to import your current resources.\n",
      });
    } else if (mode === undefined) {
      setUpdate({
        ...update,
        files_on_host: false,
        repo: "",
        file_contents: "",
      });
    }
  };

  let groups: ConfigProps<Types.ResourceSyncConfig>["groups"] = {};

  const chooseMode: ConfigGroupArgs<Types.ResourceSyncConfig> = {
    label: "Choose Mode",
    labelHidden: true,
    fields: {
      file_contents: () => {
        return (
          <ConfigItem
            label="Choose Mode"
            description="Will the file contents be defined in UI, stored on the server, or pulled from a git repo?"
          >
            <Select
              w="fit-content"
              placeholder="Choose Mode"
              value={mode}
              onChange={(mode) => mode && setMode(mode as SyncMode)}
              data={SYNC_MODES}
              disabled={disabled}
            />
          </ConfigItem>
        );
      },
    },
  };

  const generalCommon: ConfigGroupArgs<Types.ResourceSyncConfig> = {
    label: "General",
    fields: {
      delete: (delete_mode, set) => {
        return (
          <ConfigSwitch
            label="Delete Unmatched Resources"
            description="Executions will delete any resources not found in the resource files. Only use this when using one sync for everything."
            value={managed || delete_mode}
            onCheckedChange={(delete_mode) => set({ delete: delete_mode })}
            disabled={disabled || managed}
          />
        );
      },
      managed: {
        label: "Managed",
        description:
          "Enabled managed mode / the 'Commit' button. Commit is the 'reverse' of Execute, and will update the sync file with your configs updated in the UI.",
      },
    },
  };

  const includeToggles: ConfigGroupArgs<Types.ResourceSyncConfig> = {
    label: "Include",
    fields: {
      include_resources: {
        label: "Sync Resources",
        description: "Include resources (servers, stacks, etc.) in the sync.",
      },
      include_variables: {
        label: "Sync Variables",
        description: "Include variables in the sync.",
      },
      include_user_groups: {
        label: "Sync User Groups",
        description: "Include user groups in the sync.",
      },
    },
  };

  const includeResources = update.include_resources ?? config.include_resources;
  const matchTags: ConfigGroupArgs<Types.ResourceSyncConfig> = {
    label: "Match Tags",
    description: "Only sync resources matching all of these tags.",
    fields: {
      match_tags: (values, set) => {
        const tags = useRead("ListTags", {}).data;
        const otherTags = tags?.filter((tag) => !values?.includes(tag.name));
        return (
          <Group>
            <TagSelector
              title="Select Tags"
              tags={otherTags}
              onSelect={(tag) => set({ match_tags: [...(values ?? []), tag] })}
              disabled={disabled || !includeResources}
              position="bottom-start"
              useName
            />

            <Tags
              tagIds={
                tags
                  ?.filter((tag) => values?.includes(tag.name))
                  .map((tag) => tag.name) ?? []
              }
              onBadgeClick={(toRemove) =>
                set({
                  match_tags: values?.filter((tagName) => tagName !== toRemove),
                })
              }
              icon={<CircleMinus size="1rem" />}
              fz="1rem"
              useName
            />
          </Group>
        );
      },
    },
  };

  const pendingAlerts: ConfigGroupArgs<Types.ResourceSyncConfig> = {
    label: "Alerts",
    fields: {
      pending_alert: {
        label: "Pending Alerts",
        description:
          "Send a message to your Alerters when the Sync has Pending Changes",
      },
    },
  };

  if (mode === undefined) {
    groups = {
      "": [chooseMode],
    };
  } else if (mode === "Files On Server") {
    groups = {
      "": [
        {
          label: "General",
          fields: {
            resource_path: (values, set) => (
              <ConfigList
                label="Resource Paths"
                addLabel="Add Path"
                description="Add '.toml' files or folders to the sync. Relative to '/syncs/{sync_name}'."
                field="resource_path"
                values={values ?? []}
                set={set}
                disabled={disabled}
                placeholder="Input resource path"
              />
            ),
            ...generalCommon.fields,
          },
        },
        matchTags,
        includeToggles,
        pendingAlerts,
      ],
    };
  } else if (mode === "Git Repo") {
    const repoLinked = !!(update.linked_repo ?? config.linked_repo);
    const sourceConfig: ConfigGroupArgs<Types.ResourceSyncConfig> = {
      label: "Source",
      contentHidden: !show.git,
      actions: (
        <ShowHideButton
          show={show.git}
          setShow={(git) => setShow({ ...show, git })}
        />
      ),
      fields: {
        linked_repo: (linkedRepo, set) => (
          <LinkedRepo
            linkedRepo={linkedRepo}
            repoLinked={repoLinked}
            set={set}
            disabled={disabled}
          />
        ),
        ...(!repoLinked
          ? {
              git_provider: (provider: string | undefined, set) => {
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
              git_account: (value: string | undefined, set) => {
                return (
                  <AccountSelectorConfig
                    accountType="git"
                    type="None"
                    provider={update.git_provider ?? config.git_provider}
                    selected={value}
                    onSelect={(git_account) => set({ git_account })}
                    disabled={disabled}
                    placeholder="None"
                  />
                );
              },
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
            }
          : {}),
      },
    };
    const webhooksConfig: ConfigGroupArgs<Types.ResourceSyncConfig> = {
      label: "Webhooks",
      description: `Copy the webhook given here, and configure your ${webhookIntegration}-style repo provider to send webhooks to Komodo`,
      contentHidden: !show.webhooks,
      actions: (
        <ShowHideButton
          show={show.webhooks}
          setShow={(webhooks) => setShow({ ...show, webhooks })}
        />
      ),
      fields: {
        ["Guard" as any]: () => {
          if (update.branch ?? config.branch) {
            return null;
          }
          return (
            <ConfigItem label="Configure Branch">
              <Text>Must configure Branch before webhooks will work.</Text>
            </ConfigItem>
          );
        },
        ["Builder" as any]: () => <WebhookBuilder gitProvider={gitProvider} />,
        ["Refresh" as any]: () =>
          (update.branch ?? config.branch) && (
            <CopyWebhookUrl
              label="Webhook URL - Refresh Pending"
              description="Trigger an update of the pending sync cache."
              integration={webhookIntegration}
              path={`/sync/${idOrName === "Id" ? id : encodeURIComponent(name ?? "...")}/refresh`}
            />
          ),
        ["Sync" as any]: () =>
          (update.branch ?? config.branch) && (
            <CopyWebhookUrl
              label="Webhook URL - Execute Sync"
              description="Trigger an execution of the sync."
              integration={webhookIntegration}
              path={`/sync/${idOrName === "Id" ? id : encodeURIComponent(name ?? "...")}/sync`}
            />
          ),
        webhook_enabled: true,
        webhook_secret: {
          description:
            "Provide a custom webhook secret for this resource, or use the global default.",
          placeholder: "Input custom secret",
        },
      },
    };
    groups = {
      "": [
        sourceConfig,
        {
          label: "General",
          fields: {
            resource_path: (values, set) => (
              <ConfigList
                label="Resource Paths"
                addLabel="Add Path"
                description="Add '.toml' files or folders to the sync. Relative to the root of the repo."
                field="resource_path"
                values={values ?? []}
                set={set}
                disabled={disabled}
                placeholder="Input resource path"
              />
            ),
            ...generalCommon.fields,
          },
        },
        matchTags,
        includeToggles,
        pendingAlerts,
        webhooksConfig,
      ],
    };
  } else if (mode === "UI Defined") {
    groups = {
      "": [
        {
          label: "Resource File",
          description:
            "Manage the resource file contents here, or use a git repo / the files on host option.",
          actions: (
            <ShowHideButton
              show={show.file}
              setShow={(file) => setShow((show) => ({ ...show, file }))}
            />
          ),
          contentHidden: !show.file,
          fields: {
            file_contents: (file_contents, set) => {
              return (
                <MonacoEditor
                  value={
                    file_contents ||
                    "# Initialize the sync to import your current resources.\n"
                  }
                  onValueChange={(file_contents) => set({ file_contents })}
                  language="fancy_toml"
                  readOnly={disabled}
                />
              );
            },
          },
        },
        generalCommon,
        matchTags,
        includeToggles,
        pendingAlerts,
      ],
    };
  }

  return (
    <Config
      titleOther={titleOther}
      disabled={disabled}
      original={config}
      update={update}
      setUpdate={setUpdate}
      onSave={() => mutateAsync({ id, config: update })}
      groups={groups}
      fileContentsLanguage="fancy_toml"
    />
  );
}
