import { Config, ConfigComponent } from "@components/config";
import {
  AccountSelectorConfig,
  AddExtraArgMenu,
  ConfigInput,
  ConfigItem,
  ConfigList,
  ImageRegistryConfig,
  InputList,
  ProviderSelectorConfig,
  SystemCommand,
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
import { Ban, CirclePlus } from "lucide-react";
import { ReactNode } from "react";
import { CopyWebhook, ResourceLink, ResourceSelector } from "../common";
import { useToast } from "@ui/use-toast";
import { text_color_class_by_intention } from "@lib/color";
import { ConfirmButton, ShowHideButton } from "@components/util";
import { Link } from "react-router-dom";
import { SecretsSearch } from "@components/config/env_vars";
import { MonacoEditor } from "@components/monaco";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { LinkedRepoConfig } from "@components/config/linked_repo";

type BuildMode = "UI Defined" | "Files On Server" | "Git Repo" | undefined;
const BUILD_MODES: BuildMode[] = ["UI Defined", "Files On Server", "Git Repo"];

function getBuildMode(
  update: Partial<Types.BuildConfig>,
  config: Types.BuildConfig
): BuildMode {
  if (update.files_on_host ?? config.files_on_host) return "Files On Server";
  if (
    (update.repo ?? config.repo) ||
    (update.linked_repo ?? config.linked_repo)
  )
    return "Git Repo";
  if (update.dockerfile ?? config.dockerfile) return "UI Defined";
  return undefined;
}

export const BuildConfig = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) => {
  const [show, setShow] = useLocalStorage(`build-${id}-show`, {
    file: true,
    git: true,
    webhooks: true,
  });
  const { canWrite } = usePermissions({ type: "Build", id });
  const build = useRead("GetBuild", { build: id }).data;
  const config = build?.config;
  const name = build?.name;
  const webhook = useRead("GetBuildWebhookEnabled", { build: id }).data;
  const global_disabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const [update, set] = useLocalStorage<Partial<Types.BuildConfig>>(
    `build-${id}-update-v1`,
    {}
  );
  const { mutateAsync } = useWrite("UpdateBuild");
  const { integrations } = useWebhookIntegrations();
  const [id_or_name] = useWebhookIdOrName();

  if (!config) return null;

  const disabled = global_disabled || !canWrite;

  const git_provider = update.git_provider ?? config.git_provider;
  const webhook_integration = getWebhookIntegration(integrations, git_provider);

  const mode = getBuildMode(update, config);

  const setMode = (mode: BuildMode) => {
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
        dockerfile:
          update.dockerfile ||
          config.dockerfile ||
          DEFAULT_BUILD_DOCKERFILE_CONTENTS,
      });
    } else if (mode === undefined) {
      set({
        ...update,
        files_on_host: false,
        repo: "",
        dockerfile: "",
      });
    }
  };

  let components: Record<
    string,
    false | ConfigComponent<Types.BuildConfig>[] | undefined
  > = {};

  const builder_component: ConfigComponent<Types.BuildConfig> = {
    label: "Builder",
    labelHidden: true,
    components: {
      builder_id: (builder_id, set) => {
        return (
          <ConfigItem
            label={
              builder_id ? (
                <div className="flex gap-3 text-lg font-bold">
                  Builder:
                  <ResourceLink type="Builder" id={builder_id} />
                </div>
              ) : (
                "Select Builder"
              )
            }
            description="Select the Builder to build with."
            boldLabel
          >
            <ResourceSelector
              type="Builder"
              selected={builder_id}
              onSelect={(builder_id) => set({ builder_id })}
              disabled={disabled}
              align="start"
            />
          </ConfigItem>
        );
      },
    },
  };

  const version_component: ConfigComponent<Types.BuildConfig> = {
    label: "Version",
    components: {
      version: (_version, set) => {
        const version =
          typeof _version === "object"
            ? `${_version.major}.${_version.minor}.${_version.patch}`
            : _version;
        return (
          <ConfigInput
            className="text-lg w-[200px]"
            label="Version"
            description="Version the image with major.minor.patch. It can be interpolated using [[$VERSION]]."
            placeholder="0.0.0"
            value={version}
            onChange={(version) => set({ version: version as any })}
            disabled={disabled}
          />
        );
      },
      auto_increment_version: {
        description: "Automatically increment the patch number on every build.",
      },
    },
  };

  const choose_mode: ConfigComponent<Types.BuildConfig> = {
    label: "Choose Mode",
    labelHidden: true,
    components: {
      builder_id: () => {
        return (
          <ConfigItem
            label="Choose Mode"
            description="Will the dockerfile contents be defined in UI, stored on the server, or pulled from a git repo?"
            boldLabel
          >
            <Select
              value={mode}
              onValueChange={(mode) => setMode(mode as BuildMode)}
              disabled={disabled}
            >
              <SelectTrigger
                className="w-[200px] capitalize"
                disabled={disabled}
              >
                <SelectValue placeholder="Select Mode" />
              </SelectTrigger>
              <SelectContent>
                {BUILD_MODES.map((mode) => (
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

  const general_common: ConfigComponent<Types.BuildConfig>[] = [
    {
      label: "Registry",
      labelHidden: true,
      components: {
        image_registry: (registry, set) => (
          <ImageRegistryConfig
            registry={
              (registry ?? [{ domain: "", organization: "", account: "" }])[0]
            }
            setRegistry={(image_registry) =>
              set({ image_registry: [image_registry] })
            }
            resource_id={update.builder_id ?? config.builder_id}
            disabled={disabled}
          />
        ),
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

  const advanced: ConfigComponent<Types.BuildConfig>[] = [
    {
      label: "Tagging",
      components: {
        image_name: {
          description: "Push the image under a different name",
          placeholder: "Custom image name",
        },
        image_tag: {
          description: "Postfix the image version with a custom tag.",
          placeholder: "Custom image tag",
        },
      },
    },
    {
      label: "Pre Build",
      description:
        "Execute a shell command before running docker build. The 'path' is relative to the root of the repo.",
      components: {
        pre_build: (value, set) => (
          <SystemCommand
            value={value}
            set={(value) => set({ pre_build: value })}
            disabled={disabled}
          />
        ),
      },
    },
    {
      label: "Build Args",
      description:
        "Pass build args to 'docker build'. These can be used in the Dockerfile via ARG, and are visible in the final image.",
      labelExtra: !disabled && <SecretsSearch />,
      components: {
        build_args: (env, set) => (
          <MonacoEditor
            value={env || "  # VARIABLE = value\n"}
            onValueChange={(build_args) => set({ build_args })}
            language="key_value"
            readOnly={disabled}
          />
        ),
      },
    },
    {
      label: "Secret Args",
      description: (
        <div className="flex flex-row flex-wrap gap-2">
          <div>
            Pass secrets to 'docker build'. These values remain hidden in the
            final image by using docker secret mounts.
          </div>
          <Link
            to="https://docs.rs/komodo_client/latest/komodo_client/entities/build/struct.BuildConfig.html#structfield.secret_args"
            target="_blank"
            className="text-primary"
          >
            See docker docs.
          </Link>
        </div>
      ),
      labelExtra: !disabled && <SecretsSearch />,
      components: {
        secret_args: (env, set) => (
          <MonacoEditor
            value={env || "  # VARIABLE = value\n"}
            onValueChange={(secret_args) => set({ secret_args })}
            language="key_value"
            readOnly={disabled}
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
            description={
              <div className="flex flex-row flex-wrap gap-2">
                <div>Pass extra arguments to 'docker build'.</div>
                <Link
                  to="https://docs.docker.com/reference/cli/docker/buildx/build/"
                  target="_blank"
                  className="text-primary"
                >
                  See docker docs.
                </Link>
              </div>
            }
          >
            {!disabled && (
              <AddExtraArgMenu
                type="Build"
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
      label: "Labels",
      description: "Attach --labels to image.",
      components: {
        labels: (labels, set) => (
          <MonacoEditor
            value={labels || "  # your.docker.label: value\n"}
            language="key_value"
            onValueChange={(labels) => set({ labels })}
            readOnly={disabled}
          />
        ),
      },
    },
  ];

  if (mode === undefined) {
    components = {
      "": [builder_component, choose_mode],
    };
  } else if (mode === "Files On Server") {
    components = {
      "": [
        builder_component,
        version_component,
        {
          label: "Files",
          components: {
            build_path: {
              description: `Set the working directory when running the 'docker build' command. Can be absolute path, or relative to $PERIPHERY_BUILD_DIR/${build.name}`,
              placeholder: "/path/to/folder",
            },
            dockerfile_path: {
              description:
                "The path to the dockerfile, relative to the build path.",
              placeholder: "Dockerfile",
            },
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
        builder_component,
        version_component,
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
                  git_account: (account, set) => (
                    <AccountSelectorConfig
                      id={update.builder_id ?? config.builder_id ?? undefined}
                      type="Builder"
                      account_type="git"
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
          label: "Files",
          components: {
            build_path: {
              description: `The directory to run 'docker build', relative to the root of the repo.`,
              placeholder: "path/to/folder",
            },
            dockerfile_path: {
              description:
                "The path to the dockerfile, relative to the build path.",
              placeholder: "Dockerfile",
            },
          },
        },
        ...general_common,
        {
          label: "Webhook",
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
            ["build" as any]: () => (
              <ConfigItem label="Webhook Url - Build">
                <CopyWebhook
                  integration={webhook_integration}
                  path={`/build/${id_or_name === "Id" ? id : encodeURIComponent(name ?? "...")}`}
                />
              </ConfigItem>
            ),
            webhook_enabled: webhook !== undefined && !webhook.managed,
            webhook_secret: {
              description:
                "Provide a custom webhook secret for this resource, or use the global default.",
              placeholder: "Input custom secret",
            },
            ["managed" as any]: () => {
              const inv = useInvalidate();
              const { toast } = useToast();
              const { mutate: createWebhook, isPending: createPending } =
                useWrite("CreateBuildWebhook", {
                  onSuccess: () => {
                    toast({ title: "Webhook Created" });
                    inv(["GetBuildWebhookEnabled", { build: id }]);
                  },
                });
              const { mutate: deleteWebhook, isPending: deletePending } =
                useWrite("DeleteBuildWebhook", {
                  onSuccess: () => {
                    toast({ title: "Webhook Deleted" });
                    inv(["GetBuildWebhookEnabled", { build: id }]);
                  },
                });
              if (!webhook || !webhook.managed) return;
              return (
                <ConfigItem label="Manage Webhook">
                  {webhook.enabled && (
                    <div className="flex items-center gap-4 flex-wrap">
                      <div className="flex items-center gap-2">
                        Incoming webhook is{" "}
                        <div className={text_color_class_by_intention("Good")}>
                          ENABLED
                        </div>
                      </div>
                      <ConfirmButton
                        title="Disable"
                        icon={<Ban className="w-4 h-4" />}
                        variant="destructive"
                        onClick={() => deleteWebhook({ build: id })}
                        loading={deletePending}
                        disabled={disabled || deletePending}
                      />
                    </div>
                  )}
                  {!webhook.enabled && (
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
                        title="Enable Build"
                        icon={<CirclePlus className="w-4 h-4" />}
                        onClick={() => createWebhook({ build: id })}
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
        builder_component,
        version_component,
        {
          label: "Dockerfile",
          description: "Manage the dockerfile contents here.",
          contentHidden: !show.file,
          actions: (
            <ShowHideButton
              show={show.file}
              setShow={(file) => setShow({ ...show, file })}
            />
          ),
          components: {
            dockerfile: (dockerfile, set) => {
              const show_default =
                !dockerfile &&
                update.dockerfile === undefined &&
                !(update.repo ?? config.repo);
              return (
                <div className="flex flex-col gap-4">
                  <SecretsSearch />
                  <MonacoEditor
                    value={
                      show_default
                        ? DEFAULT_BUILD_DOCKERFILE_CONTENTS
                        : dockerfile
                    }
                    onValueChange={(dockerfile) => set({ dockerfile })}
                    language="dockerfile"
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
      file_contents_language="dockerfile"
    />
  );
};

export const DEFAULT_BUILD_DOCKERFILE_CONTENTS = `## Add your dockerfile here
FROM debian:stable-slim
RUN echo 'Hello Komodo'
`;
