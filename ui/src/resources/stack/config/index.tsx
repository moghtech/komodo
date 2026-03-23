import {
  usePermissions,
  useRead,
  useWebhookIdOrName,
  useWebhookIntegrations,
  useWrite,
} from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import Config, { ConfigGroupArgs, ConfigProps } from "@/ui/config";
import { ConfigItem, ConfigList, ConfigSwitch } from "@/ui/config/item";
import {
  ActionIcon,
  Button,
  Group,
  MultiSelect,
  Select,
  Stack,
  Text,
  TextInput,
} from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import ResourceLink from "@/resources/link";
import ResourceSelector from "@/resources/selector";
import ShowHideButton from "@/ui/show-hide-button";
import SecretsSearch from "@/components/config/secrets-search";
import { MonacoEditor } from "@/components/monaco";
import EnableSwitch from "@/ui/enable-switch";
import StackConfigFiles from "./config-files";
import SystemCommand from "@/components/config/system-command";
import { Link } from "react-router-dom";
import AddExtraArg from "@/components/config/add-extra-arg";
import InputList from "@/ui/input-list";
import { ProviderSelectorConfig } from "@/components/config/provider-selector";
import { AccountSelectorConfig } from "@/components/config/account-selector";
import LinkedRepo from "@/components/config/linked-repo";
import { DEFAULT_STACK_FILE_CONTENTS, useFullStack } from "..";
import { ReactNode } from "react";
import WebhookBuilder from "@/components/webhook/builder";
import CopyWebhookUrl from "@/components/webhook/copy-url";

type StackMode = "UI Defined" | "Files On Server" | "Git Repo" | undefined;
const STACK_MODES = ["UI Defined", "Files On Server", "Git Repo"] as const;

export function getStackMode(
  update: Partial<Types.StackConfig>,
  config: Types.StackConfig,
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

export default function StackConfig({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const [show, setShow] = useLocalStorage({
    key: `stack-${id}-show`,
    defaultValue: {
      file: true,
      env: true,
      webhooks: true,
    },
  });
  const { canWrite } = usePermissions({ type: "Stack", id });
  const stack = useFullStack(id);
  const allServices =
    (stack?.info?.deployed_services ?? stack?.info?.latest_services)?.map(
      (s) => s.service_name,
    ) ?? [];
  const config = stack?.config;
  const name = stack?.name;
  const global_disabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const swarmsExist = useRead("ListSwarms", {}).data?.length ? true : false;
  const [update, setUpdate] = useLocalStorage<Partial<Types.StackConfig>>({
    key: `stack-${id}-update-v1`,
    defaultValue: {},
  });
  const { mutateAsync } = useWrite("UpdateStack");
  const { getIntegration } = useWebhookIntegrations();
  const [idOrName] = useWebhookIdOrName();

  if (!config) return null;

  const disabled = global_disabled || !canWrite;

  const runBuild = update.run_build ?? config.run_build;
  const mode = getStackMode(update, config);

  const gitProvider = update.git_provider ?? config.git_provider;
  const webhookIntegration = getIntegration(gitProvider);

  const setMode = (mode: StackMode) => {
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
          DEFAULT_STACK_FILE_CONTENTS,
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

  let groups: ConfigProps<Types.StackConfig>["groups"] = {};

  const currSwarmId = update.swarm_id ?? config.swarm_id;
  const currServerId = update.server_id ?? config.server_id;

  const swarmServerGroup: ConfigGroupArgs<Types.StackConfig>[] = [
    {
      label: "Swarm",
      labelHidden: true,
      hidden: !swarmsExist || !!currServerId,
      fields: {
        swarm_id: (swarm_id, set) => {
          return (
            <ConfigItem
              label={
                swarm_id ? (
                  <Group fz="h3" fw="bold">
                    Swarm:
                    <ResourceLink
                      type="Swarm"
                      id={swarm_id}
                      fz="h3"
                      iconSize="1.2rem"
                    />
                  </Group>
                ) : (
                  "Select Swarm"
                )
              }
              description="Select the Swarm to deploy on."
            >
              <ResourceSelector
                type="Swarm"
                selected={swarm_id}
                onSelect={(swarm_id) => set({ swarm_id, server_id: "" })}
                disabled={disabled}
                clearable
              />
            </ConfigItem>
          );
        },
      },
    },
    {
      label: "Server",
      labelHidden: true,
      hidden: !!currSwarmId,
      fields: {
        server_id: (server_id, set) => {
          return (
            <ConfigItem
              label={
                server_id ? (
                  <Group fz="h3" fw="bold">
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
              description="Select the Server to deploy on."
            >
              <ResourceSelector
                type="Server"
                selected={server_id}
                onSelect={(server_id) => set({ server_id, swarm_id: "" })}
                disabled={disabled}
                clearable
              />
            </ConfigItem>
          );
        },
      },
    },
  ];

  const chooseMode: ConfigGroupArgs<Types.StackConfig> = {
    label: "Choose Mode",
    labelHidden: true,
    fields: {
      server_id: () => {
        return (
          <ConfigItem
            label="Choose Mode"
            description="Will the file contents be defined in UI, stored on the server, or pulled from a git repo?"
            boldLabel
          >
            <Select
              w="fit-content"
              placeholder="Choose Mode"
              value={mode}
              onChange={(mode) => mode && setMode(mode as StackMode)}
              disabled={disabled}
              data={STACK_MODES}
            />
          </ConfigItem>
        );
      },
    },
  };

  const environment: ConfigGroupArgs<Types.StackConfig> = {
    label: "Environment",
    hidden: !!currSwarmId,
    description: "Pass these variables to the compose command",
    actions: (
      <ShowHideButton
        show={show.env}
        setShow={(env) => setShow({ ...show, env })}
      />
    ),
    contentHidden: !show.env,
    fields: {
      environment: (env, set) => (
        <Stack>
          <SecretsSearch server={update.server_id ?? config.server_id} />
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
          "The path to write the file to, relative to the 'Run Directory'.",
        placeholder: ".env",
      },
      additional_env_files:
        (mode === "Files On Server" || mode === "Git Repo") &&
        ((values, set) => {
          const files = (values ?? []).map((v: any) =>
            typeof v === "string" ? { path: v, track: true } : v,
          );
          return (
            <ConfigItem label="Additional Env Files" boldLabel>
              <Stack>
                {files.map((file: any, i: number) => (
                  <Group key={i}>
                    <TextInput
                      value={file.path || ""}
                      onChange={(e) => {
                        set({
                          additional_env_files: files.map((v, index) =>
                            i === index
                              ? {
                                  path: e.target.value,
                                  track: file.track ?? true,
                                }
                              : v,
                          ),
                        });
                      }}
                      placeholder=".env"
                      disabled={disabled}
                      w={400}
                      maw="100%"
                    />
                    <Group>
                      <EnableSwitch
                        label="Track"
                        checked={file.track ?? true}
                        onCheckedChange={(track) => {
                          set({
                            additional_env_files: files.map((v, index) =>
                              i === index ? { ...v, track } : v,
                            ),
                          });
                        }}
                        disabled={disabled}
                        id={`track-${i}`}
                      />
                    </Group>
                    {!disabled && (
                      <ActionIcon
                        color="red"
                        onClick={() => {
                          set({
                            additional_env_files: files.filter(
                              (_: any, idx: number) => idx !== i,
                            ),
                          });
                        }}
                      >
                        <ICONS.Remove size="1rem" />
                      </ActionIcon>
                    )}
                  </Group>
                ))}
                {!disabled && (
                  <Button
                    onClick={() => {
                      set({
                        additional_env_files: [
                          ...files,
                          { path: "", track: true },
                        ],
                      });
                    }}
                    w="fit-content"
                  >
                    <ICONS.Add size="1rem" />
                    Add Env File
                  </Button>
                )}
                <Text c="dimmed" fz="sm">
                  Add additional env files to pass with '--env-file'. Relative
                  to the 'Run Directory'. Uncheck 'Track' for externally managed
                  files (e.g., sops decrypted).
                </Text>
              </Stack>
            </ConfigItem>
          );
        }),
    },
  };

  const configFiles: ConfigGroupArgs<Types.StackConfig> = {
    label: "Config Files",
    labelHidden: true,
    fields: {
      config_files: (value, set) => (
        <ConfigItem
          label="Config Files"
          description="Add other config files to associate with the Stack, and edit in the UI. Relative to 'Run Directory'."
        >
          <StackConfigFiles
            id={id}
            value={value}
            set={set}
            disabled={disabled}
          />
        </ConfigItem>
      ),
    },
  };

  const auto_update = update.auto_update ?? config.auto_update ?? false;

  const generalCommon: ConfigGroupArgs<Types.StackConfig>[] = [
    {
      label: "Auto Update",
      labelHidden: true,
      fields: {
        poll_for_updates: (poll, set) => {
          return (
            <ConfigSwitch
              label="Poll for Updates"
              description="Check for updates to the image during Global Auto Update."
              value={auto_update || poll}
              onCheckedChange={(poll_for_updates) => set({ poll_for_updates })}
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
              onCheckedChange={(auto_update_all_services) =>
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
      fields: {
        links: (values, set) => (
          <ConfigList
            label="Links"
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

  const advanced: ConfigGroupArgs<Types.StackConfig>[] = [
    {
      label: "Project Name",
      labelHidden: true,
      fields: {
        project_name: {
          placeholder: "Compose project name",
          description:
            "Optionally set a different compose project name. If importing existing stack, this should match the compose project name on your host.",
        },
      },
    },
    {
      label: "Pre Deploy",
      labelHidden: true,
      fields: {
        pre_deploy: (value, set) => (
          <ConfigItem
            label="Pre Deploy"
            description="Execute a shell command before running docker compose up. The 'path' is relative to the Run Directory"
          >
            <SystemCommand
              value={value}
              set={(value) => set({ pre_deploy: value })}
              disabled={disabled}
            />
          </ConfigItem>
        ),
      },
    },
    {
      label: "Post Deploy",
      labelHidden: true,
      fields: {
        post_deploy: (value, set) => (
          <ConfigItem
            label="Post Deploy"
            description="Execute a shell command after running docker compose up. The 'path' is relative to the Run Directory"
          >
            <SystemCommand
              value={value}
              set={(value) => set({ post_deploy: value })}
              disabled={disabled}
            />
          </ConfigItem>
        ),
      },
    },
    {
      label: "Wrapper",
      description: currSwarmId
        ? "Optional wrapper for secrets management tools. Use 'Apply To' to select which docker stack subcommands to wrap."
        : "Optional wrapper for secrets management tools. Use 'Apply To' to select which docker compose subcommands to wrap.",
      fields: {
        compose_cmd_wrapper_include: (values, set) => {
          const commands = currSwarmId
            ? ["config", "deploy"]
            : ["config", "build", "pull", "up", "run"];
          const filtered = (values ?? []).filter((v: string) =>
            commands.includes(v),
          );
          return (
            <ConfigItem label="Apply To" boldLabel>
              <MultiSelect
                placeholder={
                  filtered.length ? "Add commands" : "Select commands"
                }
                value={filtered}
                data={commands}
                onChange={(compose_cmd_wrapper_include) =>
                  set({ compose_cmd_wrapper_include })
                }
                disabled={disabled}
                w="fit-content"
                clearable
              />
            </ConfigItem>
          );
        },
        compose_cmd_wrapper: (value, set) => (
          <MonacoEditor
            value={
              value || "# sops exec-env .encrypted.env '[[COMPOSE_COMMAND]]'\n"
            }
            language="shell"
            onValueChange={(compose_cmd_wrapper) =>
              set({ compose_cmd_wrapper })
            }
            readOnly={disabled}
          />
        ),
      },
    },
    {
      label: "Extra Args",
      labelHidden: true,
      fields: {
        extra_args: (value, set) => (
          <ConfigItem
            label="Extra Args"
            description={
              <Group gap="xs">
                <Text>
                  Pass extra arguments to '
                  {currSwarmId ? "docker stack deploy" : "docker compose up"}
                  '.
                </Text>
                <Text
                  className="hover-underline"
                  fw="bold"
                  component={Link}
                  to={
                    currSwarmId
                      ? "https://docs.docker.com/reference/cli/docker/stack/deploy/#options"
                      : "https://docs.docker.com/reference/cli/docker/service/create/#options"
                  }
                  target="_blank"
                >
                  See docker docs.
                </Text>
              </Group>
            }
          >
            <InputList
              field="extra_args"
              values={value ?? []}
              set={set}
              disabled={disabled}
              placeholder="--extra-arg=value"
            />
            {!disabled && (
              <AddExtraArg
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
          </ConfigItem>
        ),
      },
    },
    {
      label: "Ignore Services",
      labelHidden: true,
      fields: {
        ignore_services: (values, set) => (
          <ConfigItem
            label="Ignore Services"
            description="If your compose file has init services that exit early, ignore them here so your stack will report the correct health."
          >
            <MultiSelect
              leftSection={<ICONS.Service size="1rem" />}
              placeholder={values?.length ? "Add services" : "Select services"}
              value={values}
              data={allServices}
              onChange={(ignore_services) => set({ ignore_services })}
              disabled={disabled}
              w="fit-content"
              searchable
              clearable
            />
          </ConfigItem>
        ),
      },
    },
    {
      label: "Pull Images",
      labelHidden: true,
      fields: {
        registry_provider: (provider, set) => {
          return (
            <ProviderSelectorConfig
              description="Login to a registry for private image access."
              accountType="docker"
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
              accountType="docker"
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
          hidden: !!currSwarmId,
          description:
            "Ensure 'docker compose pull' is run before redeploying the Stack. Otherwise, use 'pull_policy' in docker compose file.",
        },
      },
    },
    {
      label: "Build Images",
      hidden: !!currSwarmId,
      labelHidden: true,
      fields: {
        run_build: {
          label: "Pre Build Images",
          description:
            "Ensure 'docker compose build' is run before redeploying the Stack. Otherwise, can use '--build' as an Extra Arg.",
        },
        build_extra_args: (value, set) =>
          runBuild && (
            <ConfigItem
              label="Build Extra Args"
              description="Add extra args inserted after 'docker compose build'"
            >
              {!disabled && (
                <AddExtraArg
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
      fields: {
        destroy_before_deploy: {
          label: "Destroy Before Deploy",
          description: `Ensure '${
            currSwarmId ? "docker service rm" : "docker compose down"
          }' is run before redeploying the Stack.`,
        },
      },
    },
  ];

  if (mode === undefined) {
    groups = {
      "": [...swarmServerGroup, chooseMode],
    };
  } else if (mode === "Files On Server") {
    groups = {
      "": [
        ...swarmServerGroup,
        {
          label: "Files",
          labelHidden: true,
          fields: {
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
          },
        },
        environment,
        configFiles,
        ...generalCommon,
      ],
      advanced,
    };
  } else if (mode === "Git Repo") {
    const repoLinked = !!(update.linked_repo ?? config.linked_repo);
    groups = {
      "": [
        ...swarmServerGroup,
        {
          label: "Source",
          labelHidden: true,
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
                  git_account: (value, set) => {
                    const server_id = update.server_id || config.server_id;
                    return (
                      <AccountSelectorConfig
                        id={server_id}
                        type={server_id ? "Server" : "None"}
                        accountType="git"
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
                      <Stack gap="0">
                        <Text component="span">
                          Explicitly specify the folder on the host to clone the
                          repo in.
                        </Text>
                        <Text component="span">
                          If <b>relative</b> (no leading '/'), relative to{" "}
                          {"$root_directory/stacks/" + stack.name}
                        </Text>
                      </Stack>
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
          labelHidden: true,
          fields: {
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
          },
        },
        environment,
        configFiles,
        ...generalCommon,
        {
          label: "Webhooks",
          description: `Copy the webhook given here, and configure your ${webhookIntegration}-style repo provider to send webhooks to Komodo`,
          actions: (
            <ShowHideButton
              show={show.webhooks}
              setShow={(webhooks) => setShow({ ...show, webhooks })}
            />
          ),
          contentHidden: !show.webhooks,
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
            ["Builder" as any]: () => (
              <WebhookBuilder gitProvider={gitProvider} />
            ),
            ["Deploy" as any]: () =>
              (update.branch ?? config.branch) && (
                <CopyWebhookUrl
                  label="Webhook Url - Deploy"
                  integration={webhookIntegration}
                  path={`/stack/${idOrName === "Id" ? id : encodeURIComponent(name ?? "...")}/deploy`}
                />
              ),
            webhook_force_deploy: {
              description:
                "Usually the Stack won't deploy unless there are changes to the files. Use this to force deploy.",
            },
            webhook_enabled: true,
            webhook_secret: {
              description:
                "Provide a custom webhook secret for this resource, or use the global default.",
              placeholder: "Input custom secret",
            },
          },
        },
      ],
      advanced,
    };
  } else if (mode === "UI Defined") {
    groups = {
      "": [
        ...swarmServerGroup,
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
          fields: {
            file_contents: (file_contents, set) => {
              const show_default =
                !file_contents &&
                update.file_contents === undefined &&
                !(update.repo ?? config.repo);
              return (
                <Stack>
                  <SecretsSearch />
                  <MonacoEditor
                    value={
                      show_default ? DEFAULT_STACK_FILE_CONTENTS : file_contents
                    }
                    filename="compose.yaml"
                    onValueChange={(file_contents) => set({ file_contents })}
                    language="yaml"
                    readOnly={disabled}
                  />
                </Stack>
              );
            },
          },
        },
        environment,
        ...generalCommon,
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
      setUpdate={setUpdate}
      onSave={async () => {
        await mutateAsync({ id, config: update });
      }}
      groups={groups}
      fileContentsLanguage="yaml"
    />
  );
}
