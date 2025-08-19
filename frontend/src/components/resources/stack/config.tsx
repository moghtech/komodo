import { Config, ConfigComponent } from "@components/config";
import {
  AccountSelectorConfig,
  AddExtraArgMenu,
  ConfigItem,
  ConfigList,
  ConfigSwitch,
  InputList,
  ProviderSelectorConfig,
  SystemCommand,
  WebhookBuilder,
} from "@components/config/util";
import { Types } from "komodo_client";
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
import { ReactNode } from "react";
import { CopyWebhook, ResourceLink, ResourceSelector } from "../common";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { SecretsSearch } from "@components/config/env_vars";
import { ConfirmButton, ShowHideButton } from "@components/util";
import { MonacoEditor } from "@components/monaco";
import { useToast } from "@ui/use-toast";
import { text_color_class_by_intention } from "@lib/color";
import { Ban, CirclePlus } from "lucide-react";
import { LinkedRepoConfig } from "@components/config/linked_repo";

type StackMode = "UI Defined" | "Files On Server" | "Git Repo" | undefined;
const STACK_MODES: StackMode[] = ["UI Defined", "Files On Server", "Git Repo"];

function getStackMode(
  update: Partial<Types.StackConfig>,
  config: Types.StackConfig
): StackMode {
  if (update.files_on_host ?? config.files_on_host) return "Files On Server";
  if (
    (update.linked_repo ?? config.linked_repo) ||
    (update.repo ?? config.repo)
  )
    return "Git Repo";
  if (update.file_contents ?? config.file_contents) return "UI Defined";
  return undefined;
}

export const StackConfig = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) => {
  const [show, setShow] = useLocalStorage(`stack-${id}-show`, {
    file: true,
    env: true,
    git: true,
    webhooks: true,
  });
  const { canWrite } = usePermissions({ type: "Stack", id });
  const stack = useRead("GetStack", { stack: id }).data;
  const config = stack?.config;
  const name = stack?.name;
  const webhooks = useRead("GetStackWebhooksEnabled", { stack: id }).data;
  const global_disabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const [update, set] = useLocalStorage<Partial<Types.StackConfig>>(
    `stack-${id}-update-v1`,
    {}
  );
  const { mutateAsync } = useWrite("UpdateStack");
  const { integrations } = useWebhookIntegrations();
  const [id_or_name] = useWebhookIdOrName();

  if (!config) return null;

  const disabled = global_disabled || !canWrite;

  const run_build = update.run_build ?? config.run_build;
  const mode = getStackMode(update, config);

  const git_provider = update.git_provider ?? config.git_provider;
  const webhook_integration = getWebhookIntegration(integrations, git_provider);

  const setMode = (mode: StackMode) => {
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
          DEFAULT_STACK_FILE_CONTENTS,
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
    false | ConfigComponent<Types.StackConfig>[] | undefined
  > = {};

  const server_component: ConfigComponent<Types.StackConfig> = {
    label: "Server",
    labelHidden: true,
    components: {
      server_id: (server_id, set) => {
        return (
          <ConfigItem
            label={
              server_id ? (
                <div className="flex gap-3 text-lg font-bold">
                  Server:
                  <ResourceLink type="Server" id={server_id} />
                </div>
              ) : (
                "Select Server"
              )
            }
            description="Select the Server to deploy on."
          >
            <ResourceSelector
              type="Server"
              selected={server_id}
              onSelect={(server_id) => set({ server_id })}
              disabled={disabled}
              align="start"
            />
          </ConfigItem>
        );
      },
    },
  };

  const choose_mode: ConfigComponent<Types.StackConfig> = {
    label: "Choose Mode",
    labelHidden: true,
    components: {
      server_id: () => {
        return (
          <ConfigItem
            label="Choose Mode"
            description="Will the file contents be defined in UI, stored on the server, or pulled from a git repo?"
            boldLabel
          >
            <Select
              value={mode}
              onValueChange={(mode) => setMode(mode as StackMode)}
              disabled={disabled}
            >
              <SelectTrigger
                className="w-[200px] capitalize"
                disabled={disabled}
              >
                <SelectValue placeholder="Select Mode" />
              </SelectTrigger>
              <SelectContent>
                {STACK_MODES.map((mode) => (
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

  const auto_update = update.auto_update ?? config.auto_update ?? false;

  const general_common: ConfigComponent<Types.StackConfig>[] = [
    {
      label: "Environment",
      description: "Pass these variables to the compose command",
      actions: (
        <ShowHideButton
          show={show.env}
          setShow={(env) => setShow({ ...show, env })}
        />
      ),
      contentHidden: !show.env,
      components: {
        environment: (env, set) => (
          <div className="flex flex-col gap-4">
            <SecretsSearch server={update.server_id ?? config.server_id} />
            <MonacoEditor
              value={env || "  # VARIABLE = value\n"}
              onValueChange={(environment) => set({ environment })}
              language="key_value"
              readOnly={disabled}
            />
          </div>
        ),
        env_file_path: {
          description:
            "The path to write the file to, relative to the 'Run Directory'.",
          placeholder: ".env",
        },
        additional_env_files:
          (mode === "Files On Server" || mode === "Git Repo") &&
          ((values, set) => (
            <ConfigList
              label="Additional Env Files"
              boldLabel
              addLabel="Add Env File"
              description="Add additional env files to pass with '--env-file'. Relative to the 'Run Directory'."
              field="additional_env_files"
              values={values ?? []}
              set={set}
              disabled={disabled}
              placeholder=".env"
            />
          )),
      },
    },
    {
      label: "Auto Update",
      components: {
        poll_for_updates: (poll, set) => {
          return (
            <ConfigSwitch
              label="Poll for Updates"
              description="Check for updates to the image on an interval."
              value={auto_update || poll}
              onChange={(poll_for_updates) => set({ poll_for_updates })}
              disabled={disabled || auto_update}
            />
          );
        },
        auto_update: {
          description: "Trigger a redeploy if a newer image is found.",
        },
        auto_update_all_services: (value, set) => {
          return (
            <ConfigSwitch
              label="Full Stack Auto Update"
              description="Always redeploy full stack instead of just specific services with update."
              value={value}
              onChange={(auto_update_all_services) =>
                set({ auto_update_all_services })
              }
              disabled={disabled || !auto_update}
            />
          );
        },
      },
    },
    {
      label: "Links",
      labelHidden: true,
      components: {
        links: (values, set) => (
          <ConfigList
            label="Links"
            boldLabel
            addLabel="Add Link"
            description="Add quick links in the resource header"
            field="links"
            values={values ?? []}
            set={set}
            disabled={disabled}
            placeholder="Input link"
          />
        ),
      },
    },
  ];

  const advanced: ConfigComponent<Types.StackConfig>[] = [
    {
      label: "Project Name",
      labelHidden: true,
      components: {
        project_name: {
          placeholder: "Compose project name",
          boldLabel: true,
          description:
            "Optionally set a different compose project name. If importing existing stack, this should match the compose project name on your host.",
        },
      },
    },
    {
      label: "Pre Deploy",
      description:
        "Execute a shell command before running docker compose up. The 'path' is relative to the Run Directory",
      components: {
        pre_deploy: (value, set) => (
          <SystemCommand
            value={value}
            set={(value) => set({ pre_deploy: value })}
            disabled={disabled}
          />
        ),
      },
    },
    {
      label: "Post Deploy",
      description:
        "Execute a shell command after running docker compose up. The 'path' is relative to the Run Directory",
      components: {
        post_deploy: (value, set) => (
          <SystemCommand
            value={value}
            set={(value) => set({ post_deploy: value })}
            disabled={disabled}
          />
        ),
      },
    },
    {
      label: "Extra Args",
      labelHidden: true,
      components: {
        extra_args: (value, set) => (
          <ConfigItem
            label="Extra Args"
            boldLabel
            description="Add extra args inserted after 'docker compose up -d'"
          >
            {!disabled && (
              <AddExtraArgMenu
                type="Stack"
                onSelect={(suggestion) =>
                  set({
                    extra_args: [
                      ...(update.extra_args ?? config.extra_args ?? []),
                      suggestion,
                    ],
                  })
                }
                disabled={disabled}
              />
            )}
            <InputList
              field="extra_args"
              values={value ?? []}
              set={set}
              disabled={disabled}
              placeholder="--extra-arg=value"
            />
          </ConfigItem>
        ),
      },
    },
    {
      label: "Ignore Services",
      labelHidden: true,
      components: {
        ignore_services: (values, set) => (
          <ConfigList
            label="Ignore Services"
            boldLabel
            description="If your compose file has init services that exit early, ignore them here so your stack will report the correct health."
            field="ignore_services"
            values={values ?? []}
            set={set}
            disabled={disabled}
            placeholder="Input service name"
          />
        ),
      },
    },
    {
      label: "Pull Images",
      labelHidden: true,
      components: {
        registry_provider: (provider, set) => {
          return (
            <ProviderSelectorConfig
              boldLabel
              description="Login to a registry for private image access."
              account_type="docker"
              selected={provider}
              disabled={disabled}
              onSelect={(registry_provider) => set({ registry_provider })}
            />
          );
        },
        registry_account: (value, set) => {
          const server_id = update.server_id || config.server_id;
          const provider = update.registry_provider ?? config.registry_provider;
          if (!provider) {
            return null;
          }
          return (
            <AccountSelectorConfig
              id={server_id}
              type={server_id ? "Server" : "None"}
              account_type="docker"
              provider={provider}
              selected={value}
              onSelect={(registry_account) => set({ registry_account })}
              disabled={disabled}
              placeholder="None"
            />
          );
        },
        auto_pull: {
          label: "Pre Pull Images",
          description:
            "Ensure 'docker compose pull' is run before redeploying the Stack. Otherwise, use 'pull_policy' in docker compose file.",
        },
      },
    },
    {
      label: "Build Images",
      labelHidden: true,
      components: {
        run_build: {
          label: "Pre Build Images",
          description:
            "Ensure 'docker compose build' is run before redeploying the Stack. Otherwise, can use '--build' as an Extra Arg.",
        },
        build_extra_args: (value, set) =>
          run_build && (
            <ConfigItem
              label="Build Extra Args"
              description="Add extra args inserted after 'docker compose build'"
            >
              {!disabled && (
                <AddExtraArgMenu
                  type="StackBuild"
                  onSelect={(suggestion) =>
                    set({
                      build_extra_args: [
                        ...(update.build_extra_args ??
                          config.build_extra_args ??
                          []),
                        suggestion,
                      ],
                    })
                  }
                  disabled={disabled}
                />
              )}
              <InputList
                field="build_extra_args"
                values={value ?? []}
                set={set}
                disabled={disabled}
                placeholder="--extra-arg=value"
              />
            </ConfigItem>
          ),
      },
    },
    {
      label: "Destroy",
      labelHidden: true,
      components: {
        destroy_before_deploy: {
          label: "Destroy Before Deploy",
          description:
            "Ensure 'docker compose down' is run before redeploying the Stack.",
        },
      },
    },
  ];

  if (mode === undefined) {
    components = {
      "": [server_component, choose_mode],
    };
  } else if (mode === "Files On Server") {
    components = {
      "": [
        server_component,
        {
          label: "Files",
          components: {
            run_directory: {
              label: "Run Directory",
              description: `Set the working directory when running the 'compose up' command. Can be absolute path, or relative to $PERIPHERY_STACK_DIR/${stack.name}`,
              placeholder: "/path/to/folder",
            },
            file_paths: (value, set) => (
              <ConfigList
                label="File Paths"
                description="Add files to include using 'docker compose -f'. If empty, uses 'compose.yaml'. Relative to 'Run Directory'."
                field="file_paths"
                values={value ?? []}
                set={set}
                disabled={disabled}
                placeholder="compose.yaml"
              />
            ),
            additional_files: (value, set) => (
              <ConfigList
                label="Additional files"
                description="Add other config files to associate with the Stack, and edit in the UI. Relative to 'Run Directory'."
                field="additional_files"
                values={value ?? []}
                set={set}
                disabled={disabled}
                placeholder="configs/config.yaml"
              />
            ),
          },
        },
        ...general_common,
      ],
      advanced,
    };
  } else if (mode === "Git Repo") {
    const repo_linked = !!(update.linked_repo ?? config.linked_repo);
    components = {
      "": [
        server_component,
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
            linked_repo: (linked_repo, set) => (
              <LinkedRepoConfig
                linked_repo={linked_repo}
                repo_linked={repo_linked}
                set={set}
                disabled={disabled}
              />
            ),
            ...(!repo_linked
              ? {
                  git_provider: (provider, set) => {
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
                  git_account: (value, set) => {
                    const server_id = update.server_id || config.server_id;
                    return (
                      <AccountSelectorConfig
                        id={server_id}
                        type={server_id ? "Server" : "None"}
                        account_type="git"
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
                  clone_path: {
                    placeholder: "/clone/path/on/host",
                    description: (
                      <div className="flex flex-col gap-0">
                        <div>
                          Explicitly specify the folder on the host to clone the
                          repo in.
                        </div>
                        <div>
                          If <span className="font-bold">relative</span> (no
                          leading '/'), relative to{" "}
                          {"$root_directory/stacks/" + stack.name}
                        </div>
                      </div>
                    ),
                  },
                }
              : {}),
            reclone: {
              description:
                "Delete the repo folder and clone it again, instead of using 'git pull'.",
            },
          },
        },
        {
          label: "Files",
          components: {
            run_directory: {
              description:
                "Set the working directory when running the compose up command, relative to the root of the repo.",
              placeholder: "path/to/folder",
            },
            file_paths: (value, set) => (
              <ConfigList
                label="File Paths"
                description="Add files to include using 'docker compose -f'. If empty, uses 'compose.yaml'. Relative to 'Run Directory'."
                field="file_paths"
                values={value ?? []}
                set={set}
                disabled={disabled}
                placeholder="compose.yaml"
              />
            ),
            additional_files: (value, set) => (
              <ConfigList
                label="Additional files"
                description="Add other config files to associate with Stack, and edit in the UI. Relative to 'Run Directory'."
                field="additional_files"
                values={value ?? []}
                set={set}
                disabled={disabled}
                placeholder="configs/config.yaml"
              />
            ),
          },
        },
        ...general_common,
        {
          label: "Webhooks",
          description: `Copy the webhook given here, and configure your ${webhook_integration}-style repo provider to send webhooks to Komodo`,
          actions: (
            <ShowHideButton
              show={show.webhooks}
              setShow={(webhooks) => setShow({ ...show, webhooks })}
            />
          ),
          contentHidden: !show.webhooks,
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
            ["Deploy" as any]: () =>
              (update.branch ?? config.branch) && (
                <ConfigItem label="Webhook Url - Deploy">
                  <CopyWebhook
                    integration={webhook_integration}
                    path={`/stack/${id_or_name === "Id" ? id : encodeURIComponent(name ?? "...")}/deploy`}
                  />
                </ConfigItem>
              ),
            webhook_force_deploy: {
              description:
                "Usually the Stack won't deploy unless there are changes to the files. Use this to force deploy.",
            },
            webhook_enabled:
              !!(update.branch ?? config.branch) &&
              webhooks !== undefined &&
              !webhooks.managed,
            webhook_secret: {
              description:
                "Provide a custom webhook secret for this resource, or use the global default.",
              placeholder: "Input custom secret",
            },
            ["managed" as any]: () => {
              const inv = useInvalidate();
              const { toast } = useToast();
              const { mutate: createWebhook, isPending: createPending } =
                useWrite("CreateStackWebhook", {
                  onSuccess: () => {
                    toast({ title: "Webhook Created" });
                    inv(["GetStackWebhooksEnabled", { stack: id }]);
                  },
                });
              const { mutate: deleteWebhook, isPending: deletePending } =
                useWrite("DeleteStackWebhook", {
                  onSuccess: () => {
                    toast({ title: "Webhook Deleted" });
                    inv(["GetStackWebhooksEnabled", { stack: id }]);
                  },
                });

              if (
                !(update.branch ?? config.branch) ||
                !webhooks ||
                !webhooks.managed
              ) {
                return null;
              }

              return (
                <ConfigItem label="Manage Webhook">
                  {webhooks.deploy_enabled && (
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
                          DEPLOY
                        </div>
                      </div>
                      <ConfirmButton
                        title="Disable"
                        icon={<Ban className="w-4 h-4" />}
                        variant="destructive"
                        onClick={() =>
                          deleteWebhook({
                            stack: id,
                            action: Types.StackWebhookAction.Deploy,
                          })
                        }
                        loading={deletePending}
                        disabled={disabled || deletePending}
                      />
                    </div>
                  )}
                  {!webhooks.deploy_enabled && webhooks.refresh_enabled && (
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
                          REFRESH
                        </div>
                      </div>
                      <ConfirmButton
                        title="Disable"
                        icon={<Ban className="w-4 h-4" />}
                        variant="destructive"
                        onClick={() =>
                          deleteWebhook({
                            stack: id,
                            action: Types.StackWebhookAction.Refresh,
                          })
                        }
                        loading={deletePending}
                        disabled={disabled || deletePending}
                      />
                    </div>
                  )}
                  {!webhooks.deploy_enabled && !webhooks.refresh_enabled && (
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
                        title="Enable Deploy"
                        icon={<CirclePlus className="w-4 h-4" />}
                        onClick={() =>
                          createWebhook({
                            stack: id,
                            action: Types.StackWebhookAction.Deploy,
                          })
                        }
                        loading={createPending}
                        disabled={disabled || createPending}
                      />
                      <ConfirmButton
                        title="Enable Refresh"
                        icon={<CirclePlus className="w-4 h-4" />}
                        onClick={() =>
                          createWebhook({
                            stack: id,
                            action: Types.StackWebhookAction.Refresh,
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
      advanced,
    };
  } else if (mode === "UI Defined") {
    components = {
      "": [
        server_component,
        {
          label: "Compose File",
          description: "Manage the compose file contents here.",
          actions: (
            <ShowHideButton
              show={show.file}
              setShow={(file) => setShow({ ...show, file })}
            />
          ),
          contentHidden: !show.file,
          components: {
            file_contents: (file_contents, set) => {
              const show_default =
                !file_contents &&
                update.file_contents === undefined &&
                !(update.repo ?? config.repo);
              return (
                <div className="flex flex-col gap-4">
                  <SecretsSearch />
                  <MonacoEditor
                    value={
                      show_default ? DEFAULT_STACK_FILE_CONTENTS : file_contents
                    }
                    onValueChange={(file_contents) => set({ file_contents })}
                    language="yaml"
                    readOnly={disabled}
                  />
                </div>
              );
            },
          },
        },
        ...general_common,
      ],
      advanced,
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
      file_contents_language="yaml"
    />
  );
};

export const DEFAULT_STACK_FILE_CONTENTS = `## Add your compose file here
services:
  hello_world:
    image: hello-world
    # networks:
    #   - default
    # ports:
    #   - 3000:3000
    # volumes:
    #   - data:/data

# networks:
#   default: {}

# volumes:
#   data:
`;
