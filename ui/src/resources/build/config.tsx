import {
  getWebhookIntegration,
  usePermissions,
  useRead,
  useWebhookIdOrName,
  useWebhookIntegrations,
  useWrite,
} from "@/lib/hooks";
import { ConfigGroupArgs, ConfigProps } from "@/ui/config";
import { ConfigInput, ConfigItem } from "@/ui/config/item";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import ResourceSelector from "@/resources/selector";
import { ResourceLink } from "@/resources/common";
import { Button, Group, Select } from "@mantine/core";
import { ICONS } from "@/lib/icons";

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

export default function BuildConfig({ id }: { id: string }) {
  const [show, setShow] = useLocalStorage({
    key: `build-${id}-show`,
    defaultValue: {
      file: true,
      git: true,
      webhooks: true,
    },
  });
  const { canWrite } = usePermissions({ type: "Build", id });
  const build = useRead("GetBuild", { build: id }).data;
  const config = build?.config;
  const name = build?.name;
  const global_disabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const [update, set] = useLocalStorage<Partial<Types.BuildConfig>>({
    key: `build-${id}-update-v1`,
    defaultValue: {},
  });
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

  let groups: ConfigProps<Types.BuildConfig>["groups"] = {};

  const builderGroup: ConfigGroupArgs<Types.BuildConfig> = {
    label: "Builder",
    labelHidden: true,
    fields: {
      builder_id(builder_id, set) {
        return (
          <ConfigItem
            label={
              builder_id ? (
                <Group>
                  Builder:
                  <ResourceLink type="Builder" id={builder_id} />
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
              selected={builder_id}
              onSelect={(builder_id) => set({ builder_id })}
              disabled={disabled}
              position="bottom-start"
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
            fz="lg"
            w={200}
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
          >
            {!disabled && (
              <Button
                variant="light"
                onClick={() =>
                  set({
                    image_registry: [
                      ...(image_registries ?? []),
                      { domain: "", organization: "", account: "" },
                    ],
                  })
                }
                className="flex items-center gap-2 w-[200px]"
              >
                <ICONS.Create size="1rem" />
                Add Registry
              </Button>
            )}

            {/* {image_registries?.map((registry, index) => (
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
                builder_id={update.builder_id ?? config.builder_id}
                disabled={disabled}
              />
            ))} */}
          </ConfigItem>
        ),
      },
    },
  ];
}
