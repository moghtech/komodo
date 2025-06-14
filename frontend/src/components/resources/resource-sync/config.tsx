import { Config, ConfigComponent } from "@components/config";
import {
  AccountSelectorConfig,
  ConfigItem,
  ConfigList,
  ConfigSwitch,
  ProviderSelectorConfig,
  WebhookBuilder,
} from "@components/config/util";
import {
  getWebhookIntegration,
  useInvalidate,
  useLocalStorage,
  usePermissions,
  useRead,
  useWebhookIdOrName,
  useWebhookIntegrations,
  useWrite,
} from "@lib/hooks";
import { Types } from "komodo_client";
import { ReactNode, useState } from "react";
import { CopyWebhook, ResourceLink, ResourceSelector } from "../common";
import { useToast } from "@ui/use-toast";
import { text_color_class_by_intention } from "@lib/color";
import { ConfirmButton, ShowHideButton } from "@components/util";
import { Ban, CirclePlus, MinusCircle, SearchX, Tag } from "lucide-react";
import { MonacoEditor } from "@components/monaco";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { filterBySplit } from "@lib/utils";
import { Button } from "@ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@ui/popover";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@ui/command";

type SyncMode = "UI Defined" | "Files On Server" | "Git Repo" | undefined;
const SYNC_MODES: SyncMode[] = ["UI Defined", "Files On Server", "Git Repo"];

function getSyncMode(
  update: Partial<Types.ResourceSyncConfig>,
  config: Types.ResourceSyncConfig
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

export const ResourceSyncConfig = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) => {
  const [show, setShow] = useLocalStorage(`sync-${id}-show`, {
    file: true,
    git: true,
    webhooks: true,
  });
  const { canWrite } = usePermissions({ type: "ResourceSync", id });
  const sync = useRead("GetResourceSync", { sync: id }).data;
  const config = sync?.config;
  const name = sync?.name;
  const webhooks = useRead("GetSyncWebhooksEnabled", { sync: id }).data;
  const global_disabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const [update, set] = useLocalStorage<Partial<Types.ResourceSyncConfig>>(
    `sync-${id}-update-v1`,
    {}
  );
  const { mutateAsync } = useWrite("UpdateResourceSync");
  const { integrations } = useWebhookIntegrations();
  const [id_or_name] = useWebhookIdOrName();

  if (!config) return null;

  const disabled = global_disabled || !canWrite;

  const git_provider = update.git_provider ?? config.git_provider;
  const webhook_integration = getWebhookIntegration(integrations, git_provider);

  const mode = getSyncMode(update, config);
  const managed = update.managed ?? config.managed ?? false;

  const setMode = (mode: SyncMode) => {
    if (mode === "Files On Server") {
      set({ ...update, files_on_host: true });
    } else if (mode === "Git Repo") {
      set({
        ...update,
        files_on_host: false,
        repo: update.repo || config.repo || "namespace/repo",
      });
    } else if (mode === "UI Defined") {
      set({
        ...update,
        files_on_host: false,
        repo: "",
        file_contents:
          update.file_contents ||
          config.file_contents ||
          "# Initialize the sync to import your current resources.\n",
      });
    } else if (mode === undefined) {
      set({
        ...update,
        files_on_host: false,
        repo: "",
        file_contents: "",
      });
    }
  };

  let components: Record<
    string,
    false | ConfigComponent<Types.ResourceSyncConfig>[] | undefined
  > = {};

  const choose_mode: ConfigComponent<Types.ResourceSyncConfig> = {
    label: "Choose Mode",
    labelHidden: true,
    components: {
      file_contents: () => {
        return (
          <ConfigItem
            label="Choose Mode"
            description="Will the file contents be defined in UI, stored on the server, or pulled from a git repo?"
            boldLabel
          >
            <Select
              value={mode}
              onValueChange={(mode) => setMode(mode as SyncMode)}
              disabled={disabled}
            >
              <SelectTrigger
                className="w-[200px] capitalize"
                disabled={disabled}
              >
                <SelectValue placeholder="Select Mode" />
              </SelectTrigger>
              <SelectContent>
                {SYNC_MODES.map((mode) => (
                  <SelectItem
                    key={mode}
                    value={mode!}
                    className="capitalize cursor-pointer"
                  >
                    {mode}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </ConfigItem>
        );
      },
    },
  };

  const general_common: ConfigComponent<Types.ResourceSyncConfig> = {
    label: "General",
    components: {
      delete: (delete_mode, set) => {
        return (
          <ConfigSwitch
            label="Delete Unmatched Resources"
            description="Executions will delete any resources not found in the resource files. Only use this when using one sync for everything."
            value={managed || delete_mode}
            onChange={(delete_mode) => set({ delete: delete_mode })}
            disabled={disabled || managed}
          />
        );
      },
      managed: {
        label: "Managed",
        description:
          "Enabled managed mode / the 'Commit' button. Commit is the 'reverse' of Execute, and will update the sync file with your configs updated in the UI.",
      },
      pending_alert: {
        label: "Pending Alerts",
        description:
          "Send a message to your Alerters when the Sync has Pending Changes",
      },
    },
  };

  const include_toggles: ConfigComponent<Types.ResourceSyncConfig> = {
    label: "Include",
    components: {
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

  const include_resources =
    update.include_resources ?? config.include_resources;
  const match_tags: ConfigComponent<Types.ResourceSyncConfig> = {
    label: "Match Tags",
    description: "Only sync resources matching all of these tags.",
    components: {
      match_tags: (values, set) => (
        <MatchTags
          tags={values ?? []}
          set={set}
          disabled={disabled || !include_resources}
        />
      ),
    },
  };

  if (mode === undefined) {
    components = {
      "": [choose_mode],
    };
  } else if (mode === "Files On Server") {
    components = {
      "": [
        {
          label: "General",
          components: {
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
            ...general_common.components,
          },
        },
        include_toggles,
        match_tags,
      ],
    };
  } else if (mode === "Git Repo") {
    const repo_linked = !!(update.linked_repo ?? config.linked_repo);
    const linked_repo_component = (
      linked_repo: string | undefined,
      set: (config: Partial<Types.ResourceSyncConfig>) => void
    ) => {
      return (
        <ConfigItem
          label={
            linked_repo ? (
              <div className="flex gap-3 text-lg font-bold">
                Repo:
                <ResourceLink type="Repo" id={linked_repo} />
              </div>
            ) : (
              "Select Repo"
            )
          }
          description={`Select an existing Repo to attach${!repo_linked ? ", or configure the repo below" : ""}.`}
        >
          <ResourceSelector
            type="Repo"
            selected={linked_repo}
            onSelect={(linked_repo) =>
              set({
                linked_repo,
                // Set other props back to default.
                git_provider: "github.com",
                git_account: "",
                git_https: true,
                repo: linked_repo ? "" : "namespace/repo",
                branch: "main",
                commit: "",
              })
            }
            disabled={disabled}
            align="start"
          />
        </ConfigItem>
      );
    };
    components = {
      "": [
        {
          label: "Source",
          contentHidden: !show.git,
          actions: (
            <ShowHideButton
              show={show.git}
              setShow={(git) => setShow({ ...show, git })}
            />
          ),
          components: {
            linked_repo: linked_repo_component,
            ...(!repo_linked
              ? {
                  git_provider: (provider: string | undefined, set) => {
                    const https = update.git_https ?? config.git_https;
                    return (
                      <ProviderSelectorConfig
                        account_type="git"
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
                        account_type="git"
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
                    description:
                      "Select a custom branch, or default to 'main'.",
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
        },
        {
          label: "General",
          components: {
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
            ...general_common.components,
          },
        },
        include_toggles,
        match_tags,
        {
          label: "Git Webhooks",
          description: `Copy the webhook given here, and configure your ${webhook_integration}-style repo provider to send webhooks to Komodo`,
          contentHidden: !show.webhooks,
          actions: (
            <ShowHideButton
              show={show.webhooks}
              setShow={(webhooks) => setShow({ ...show, webhooks })}
            />
          ),
          components: {
            ["Guard" as any]: () => {
              if (update.branch ?? config.branch) {
                return null;
              }
              return (
                <ConfigItem label="Configure Branch">
                  <div>Must configure Branch before webhooks will work.</div>
                </ConfigItem>
              );
            },
            ["Builder" as any]: () => (
              <WebhookBuilder git_provider={git_provider} />
            ),
            ["Refresh" as any]: () => (
              <ConfigItem
                label="Webhook Url - Refresh Pending"
                description="Trigger an update of the pending sync cache, to display the changes in the UI on push."
              >
                <CopyWebhook
                  integration={webhook_integration}
                  path={`/sync/${id_or_name === "Id" ? id : encodeURIComponent(name ?? "...")}/refresh`}
                />
              </ConfigItem>
            ),
            ["Sync" as any]: () => (
              <ConfigItem
                label="Webhook Url - Execute Sync"
                description="Trigger an execution of the sync on push."
              >
                <CopyWebhook
                  integration={webhook_integration}
                  path={`/sync/${id_or_name === "Id" ? id : encodeURIComponent(name ?? "...")}/sync`}
                />
              </ConfigItem>
            ),
            webhook_enabled: webhooks !== undefined && !webhooks.managed,
            webhook_secret: {
              description:
                "Provide a custom webhook secret for this resource, or use the global default.",
              placeholder: "Input custom secret",
            },
            ["managed" as any]: () => {
              const inv = useInvalidate();
              const { toast } = useToast();
              const { mutate: createWebhook, isPending: createPending } =
                useWrite("CreateSyncWebhook", {
                  onSuccess: () => {
                    toast({ title: "Webhook Created" });
                    inv(["GetSyncWebhooksEnabled", { sync: id }]);
                  },
                });
              const { mutate: deleteWebhook, isPending: deletePending } =
                useWrite("DeleteSyncWebhook", {
                  onSuccess: () => {
                    toast({ title: "Webhook Deleted" });
                    inv(["GetSyncWebhooksEnabled", { sync: id }]);
                  },
                });
              if (!webhooks || !webhooks.managed) return;
              return (
                <ConfigItem label="Manage Webhook">
                  {webhooks.sync_enabled && (
                    <div className="flex items-center gap-4 flex-wrap">
                      <div className="flex items-center gap-2">
                        Incoming webhook is{" "}
                        <div className={text_color_class_by_intention("Good")}>
                          ENABLED
                        </div>
                        and will trigger
                        <div
                          className={text_color_class_by_intention("Neutral")}
                        >
                          SYNC EXECUTION
                        </div>
                      </div>
                      <ConfirmButton
                        title="Disable"
                        icon={<Ban className="w-4 h-4" />}
                        variant="destructive"
                        onClick={() =>
                          deleteWebhook({
                            sync: id,
                            action: Types.SyncWebhookAction.Sync,
                          })
                        }
                        loading={deletePending}
                        disabled={disabled || deletePending}
                      />
                    </div>
                  )}
                  {!webhooks.sync_enabled && webhooks.refresh_enabled && (
                    <div className="flex items-center gap-4 flex-wrap">
                      <div className="flex items-center gap-2">
                        Incoming webhook is{" "}
                        <div className={text_color_class_by_intention("Good")}>
                          ENABLED
                        </div>
                        and will trigger
                        <div
                          className={text_color_class_by_intention("Neutral")}
                        >
                          PENDING REFRESH
                        </div>
                      </div>
                      <ConfirmButton
                        title="Disable"
                        icon={<Ban className="w-4 h-4" />}
                        variant="destructive"
                        onClick={() =>
                          deleteWebhook({
                            sync: id,
                            action: Types.SyncWebhookAction.Refresh,
                          })
                        }
                        loading={deletePending}
                        disabled={disabled || deletePending}
                      />
                    </div>
                  )}
                  {!webhooks.sync_enabled && !webhooks.refresh_enabled && (
                    <div className="flex items-center gap-4 flex-wrap">
                      <div className="flex items-center gap-2">
                        Incoming webhook is{" "}
                        <div
                          className={text_color_class_by_intention("Critical")}
                        >
                          DISABLED
                        </div>
                      </div>
                      <ConfirmButton
                        title="Enable Refresh"
                        icon={<CirclePlus className="w-4 h-4" />}
                        onClick={() =>
                          createWebhook({
                            sync: id,
                            action: Types.SyncWebhookAction.Refresh,
                          })
                        }
                        loading={createPending}
                        disabled={disabled || createPending}
                      />
                      <ConfirmButton
                        title="Enable Sync"
                        icon={<CirclePlus className="w-4 h-4" />}
                        onClick={() =>
                          createWebhook({
                            sync: id,
                            action: Types.SyncWebhookAction.Sync,
                          })
                        }
                        loading={createPending}
                        disabled={disabled || createPending}
                      />
                    </div>
                  )}
                </ConfigItem>
              );
            },
          },
        },
      ],
    };
  } else if (mode === "UI Defined") {
    components = {
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
          components: {
            file_contents: (file_contents, set) => {
              return (
                <MonacoEditor
                  value={
                    file_contents ||
                    "# Initialize the sync to import your current resources.\n"
                  }
                  onValueChange={(file_contents) => set({ file_contents })}
                  language="toml"
                  readOnly={disabled}
                />
              );
            },
          },
        },
        general_common,
        include_toggles,
        match_tags,
      ],
    };
  }

  return (
    <Config
      titleOther={titleOther}
      disabled={disabled}
      original={config}
      update={update}
      set={set}
      onSave={async () => {
        await mutateAsync({ id, config: update });
      }}
      components={components}
      file_contents_language="toml"
    />
  );
};

const MatchTags = ({
  tags,
  set,
  disabled,
}: {
  tags: string[];
  set: (update: Partial<Types.ResourceSyncConfig>) => void;
  disabled: boolean;
}) => {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const all_tags = useRead("ListTags", {}).data;
  const filtered = filterBySplit(all_tags, search, (item) => item.name);
  return (
    <div className="flex gap-3 items-center">
      <Popover
        open={open}
        onOpenChange={(open) => {
          setSearch("");
          setOpen(open);
        }}
      >
        <PopoverTrigger asChild>
          <Button
            variant="outline"
            className="flex items-center gap-2"
            disabled={disabled}
          >
            <Tag className="w-3 h-3" />
            Select Tag
          </Button>
        </PopoverTrigger>
        <PopoverContent
          className="w-[200px] max-h-[200px] p-0"
          sideOffset={12}
          align="start"
        >
          <Command shouldFilter={false}>
            <CommandInput
              placeholder="Search Tags"
              className="h-9"
              value={search}
              onValueChange={setSearch}
            />
            <CommandList>
              <CommandEmpty className="flex justify-evenly items-center pt-2">
                No Tags Found
                <SearchX className="w-3 h-3" />
              </CommandEmpty>

              <CommandGroup>
                {filtered
                  ?.filter((tag) => !tags.includes(tag.name))
                  .map((tag) => (
                    <CommandItem
                      key={tag.name}
                      onSelect={() => {
                        set({ match_tags: [...tags, tag.name] });
                        setSearch("");
                        setOpen(false);
                      }}
                      className="flex items-center justify-between cursor-pointer"
                    >
                      <div className="p-1">{tag.name}</div>
                    </CommandItem>
                  ))}
              </CommandGroup>
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>

      <MatchTagsTags
        tags={tags}
        onBadgeClick={(tag) =>
          set({ match_tags: tags.filter((name) => name !== tag) })
        }
        disabled={disabled}
      />
    </div>
  );
};

const MatchTagsTags = ({
  tags,
  onBadgeClick,
  disabled,
}: {
  tags?: string[];
  onBadgeClick: (tag: string) => void;
  disabled: boolean;
}) => {
  return (
    <>
      {tags?.map((tag) => (
        <Button
          key={tag}
          variant="secondary"
          className="flex items-center gap-2"
          onClick={() => onBadgeClick && onBadgeClick(tag)}
          disabled={disabled}
        >
          {tag}
          <MinusCircle className="w-4 h-4" />
        </Button>
      ))}
    </>
  );
};
