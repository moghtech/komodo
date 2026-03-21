import {
  usePermissions,
  useRead,
  useWebhookIdOrName,
  useWebhookIntegrations,
  useWrite,
} from "@/lib/hooks";
import Config, { ConfigGroupArgs, ConfigProps } from "@/ui/config";
import { ConfigInput, ConfigItem, ConfigList } from "@/ui/config/item";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import ResourceSelector from "@/resources/selector";
import ResourceLink from "@/resources/link";
import { Button, Group, Select, Stack, Text } from "@mantine/core";
import { ICONS } from "@/theme/icons";
import ImageRegistryConfig from "@/components/config/image-registry-config";
import SystemCommand from "@/components/config/system-command";
import { MonacoEditor } from "@/components/monaco";
import SecretsSearch from "@/components/config/secrets-search";
import { Link } from "react-router-dom";
import AddExtraArg from "@/components/config/add-extra-arg";
import InputList from "@/ui/input-list";
import ShowHideButton from "@/ui/show-hide-button";
import LinkedRepo from "@/components/config/linked-repo";
import { ProviderSelectorConfig } from "@/components/config/provider-selector";
import { AccountSelectorConfig } from "@/components/config/account-selector";
import { ReactNode } from "react";
import WebhookBuilder from "@/components/webhook/builder";
import CopyWebhookUrl from "@/components/webhook/copy-url";
import { useFullBuild } from ".";

type BuildMode = "UI Defined" | "Files On Server" | "Git Repo" | undefined;
const BUILD_MODES = ["UI Defined", "Files On Server", "Git Repo"] as const;

function getBuildMode(
  update: Partial<Types.BuildConfig>,
  config: Types.BuildConfig,
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

export const DEFAULT_BUILD_DOCKERFILE_CONTENTS = `## Add your dockerfile here
FROM debian:stable-slim
RUN echo 'Hello Komodo'
`;

export default function BuildConfig({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const [show, setShow] = useLocalStorage({
    key: `build-${id}-show`,
    defaultValue: {
      file: true,
      git: true,
      webhooks: true,
    },
  });
  const { canWrite } = usePermissions({ type: "Build", id });
  const build = useFullBuild(id);
  const config = build?.config;
  const name = build?.name;
  const globalDisabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const [update, setUpdate] = useLocalStorage<Partial<Types.BuildConfig>>({
    key: `build-${id}-update-v1`,
    defaultValue: {},
  });
  const { mutateAsync } = useWrite("UpdateBuild");
  const { getIntegration } = useWebhookIntegrations();
  const [idOrName] = useWebhookIdOrName();

  if (!config) return null;

  const disabled = globalDisabled || !canWrite;

  const gitProvider = update.git_provider ?? config.git_provider;
  const webhookIntegration = getIntegration(gitProvider);

  const mode = getBuildMode(update, config);

  const setMode = (mode: BuildMode) => {
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
        dockerfile:
          update.dockerfile ||
          config.dockerfile ||
          DEFAULT_BUILD_DOCKERFILE_CONTENTS,
      });
    } else if (mode === undefined) {
      setUpdate({
        ...update,
        files_on_host: false,
        repo: "",
        dockerfile: "",
      });
    }
  };

  let groups: ConfigProps<Types.BuildConfig>["groups"] = {};

  const builderGroup: ConfigGroupArgs<Types.BuildConfig> = {
    label: "Builder",
    labelHidden: true,
    fields: {
      builder_id(builderId, set) {
        return (
          <ConfigItem
            label={
              builderId ? (
                <Group fz="h3" fw="bold">
                  Builder:
                  <ResourceLink
                    type="Builder"
                    id={builderId}
                    fz="h3"
                    iconSize="1.2rem"
                  />
                </Group>
              ) : (
                "Select Builder"
              )
            }
            description="Select the Builder to build with."
            boldLabel
          >
            <ResourceSelector
              type="Builder"
              selected={builderId}
              onSelect={(builder_id) => set({ builder_id })}
              disabled={disabled}
              clearable
            />
          </ConfigItem>
        );
      },
    },
  };

  const versionGroup: ConfigGroupArgs<Types.BuildConfig> = {
    label: "Version",
    labelHidden: true,
    fields: {
      version: (_version, set) => {
        const version =
          typeof _version === "object"
            ? `${_version.major}.${_version.minor}.${_version.patch}`
            : _version;
        return (
          <ConfigInput
            inputProps={{ size: "lg" }}
            label="Version"
            boldLabel
            description="Version the image with major.minor.patch. It can be interpolated using [[$VERSION]]."
            placeholder="0.0.0"
            value={version}
            onValueChange={(version) => set({ version: version as any })}
            disabled={disabled}
          />
        );
      },
      auto_increment_version: {
        description: "Automatically increment the patch number on every build.",
      },
    },
  };

  const chooseMode: ConfigGroupArgs<Types.BuildConfig> = {
    label: "Choose Mode",
    labelHidden: true,
    fields: {
      builder_id: () => {
        return (
          <ConfigItem
            label="Choose Mode"
            description="Will the dockerfile contents be defined in UI, stored on the server, or pulled from a git repo?"
            boldLabel
          >
            <Select
              w="fit-content"
              placeholder="Choose Mode"
              value={mode}
              onChange={(mode) => mode && setMode(mode as BuildMode)}
              data={BUILD_MODES}
              disabled={disabled}
            />
          </ConfigItem>
        );
      },
    },
  };

  const imageName = (update.image_name ?? config.image_name) || name;
  const customTag = update.image_tag ?? config.image_tag;
  const customTagPostfix = customTag ? `-${customTag}` : "";

  const generalCommon: ConfigGroupArgs<Types.BuildConfig>[] = [
    {
      label: "Registry",
      labelHidden: true,
      fields: {
        image_registry: (image_registries, set) => (
          <ConfigItem
            label="Image Registry"
            boldLabel
            description="Configure where the built image is pushed."
            gap="xl"
          >
            {!disabled && (
              <Button
                onClick={() =>
                  set({
                    image_registry: [
                      ...(image_registries ?? []),
                      { domain: "", organization: "", account: "" },
                    ],
                  })
                }
                w={200}
              >
                <ICONS.Create size="1rem" />
                Add Registry
              </Button>
            )}

            {image_registries?.map((registry, index) => (
              <ImageRegistryConfig
                key={
                  (registry.domain ?? "") +
                  (registry.organization ?? "") +
                  (registry.account ?? "") +
                  index
                }
                registry={registry}
                imageName={imageName}
                setRegistry={(registry) =>
                  set({
                    image_registry:
                      image_registries?.map((r, i) =>
                        i === index ? registry : r,
                      ) ?? [],
                  })
                }
                onRemove={() =>
                  set({
                    image_registry:
                      image_registries?.filter((_, i) => i !== index) ?? [],
                  })
                }
                builderId={update.builder_id ?? config.builder_id}
                disabled={disabled}
              />
            ))}
          </ConfigItem>
        ),
      },
    },
    {
      label: "Tagging",
      labelHidden: true,
      fields: {
        image_name: {
          description: "Push the image under a different name",
          placeholder: "Custom image name",
        },
        image_tag: {
          description: `Push a custom tag, plus postfix the other tags (eg ':latest-${customTag ? customTag : "<TAG>"}').`,
          placeholder: "Custom image tag",
        },
        include_latest_tag: {
          description: `:latest${customTagPostfix}`,
        },
        include_version_tags: {
          description: `:X.Y.Z${customTagPostfix} + :X.Y${customTagPostfix} + :X${customTagPostfix}`,
        },
        include_commit_tag: {
          description: `:ae8f8ff${customTagPostfix}`,
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

  const advanced: ConfigGroupArgs<Types.BuildConfig>[] = [
    {
      label: "Pre Build",
      description:
        "Execute a shell command before running docker build. The 'path' is relative to the root of the repo.",
      fields: {
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
      labelExtra: !disabled && (
        <SecretsSearch builder={update.builder_id ?? config.builder_id} />
      ),
      fields: {
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
        <Group>
          <Text>
            Pass secrets to 'docker build'. These values remain hidden in the
            final image by using docker secret mounts.
          </Text>
          <Link
            to="https://docs.rs/komodo_client/latest/komodo_client/entities/build/struct.BuildConfig.html#structfield.secret_args"
            target="_blank"
            className="text-primary"
          >
            See docker docs.
          </Link>
        </Group>
      ),
      labelExtra: !disabled && <SecretsSearch />,
      fields: {
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
      fields: {
        extra_args: (value, set) => (
          <ConfigItem
            label="Extra Args"
            boldLabel
            description={
              <Group gap="xs">
                <Text>Pass extra arguments to 'docker build'.</Text>
                <Text
                  className="hover-underline"
                  fw="bold"
                  component={Link}
                  to="https://docs.docker.com/reference/cli/docker/buildx/build/"
                  target="_blank"
                >
                  See docker docs.
                </Text>
              </Group>
            }
          >
            {!disabled && (
              <AddExtraArg
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
      fields: {
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
    groups = {
      "": [builderGroup, chooseMode],
    };
  } else if (mode === "Files On Server") {
    groups = {
      "": [
        builderGroup,
        versionGroup,
        {
          label: "Files",
          fields: {
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
        ...generalCommon,
      ],
      advanced,
    };
  } else if (mode === "Git Repo") {
    const repoLinked = !!(update.linked_repo ?? config.linked_repo);
    groups = {
      "": [
        builderGroup,
        versionGroup,
        {
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
                      type="Builder"
                      accountType="git"
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
          fields: {
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
        ...generalCommon,
        {
          label: "Webhook",
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
            ["Builder" as any]: () => (
              <WebhookBuilder gitProvider={gitProvider} />
            ),
            ["Build" as any]: () =>
              (update.branch ?? config.branch) && (
                <CopyWebhookUrl
                  label="Webhook Url - Build"
                  integration={webhookIntegration}
                  path={`/build/${idOrName === "Id" ? id : encodeURIComponent(name ?? "...")}`}
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
      ],
      advanced,
    };
  } else if (mode === "UI Defined") {
    groups = {
      "": [
        builderGroup,
        versionGroup,
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
          fields: {
            dockerfile: (dockerfile, set) => {
              const show_default =
                !dockerfile &&
                update.dockerfile === undefined &&
                !(update.repo ?? config.repo);
              return (
                <Stack>
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
                </Stack>
              );
            },
          },
        },
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
      onSave={() => mutateAsync({ id, config: update })}
      groups={groups}
      fileContentsLanguage="dockerfile"
    />
  );
}
