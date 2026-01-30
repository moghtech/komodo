import {
  getWebhookIntegration,
  usePermissions,
  useRead,
  useWebhookIdOrName,
  useWebhookIntegrations,
  useWrite,
} from "@/lib/hooks";
import { ConfigGroupArgs, ConfigProps } from "@/ui/config";
import { ConfigItem } from "@/ui/config/item";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import ResourceSelector from "@/resources/selector";
import { ResourceLink } from "@/resources/common";

type BuildMode = "UI Defined" | "Files On Server" | "Git Repo" | undefined;
const BUILD_MODES: BuildMode[] = ["UI Defined", "Files On Server", "Git Repo"];

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
              position="bottom-start"
            />
          </ConfigItem>
        );
      },
    },
  };
}
