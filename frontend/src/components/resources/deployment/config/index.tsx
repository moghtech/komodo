import { useLocalStorage, usePermissions, useRead, useWrite } from "@lib/hooks";
import { Types } from "komodo_client";
import { ReactNode } from "react";
import {
  AccountSelectorConfig,
  AddExtraArgMenu,
  ConfigItem,
  ConfigList,
  ConfigSwitch,
  InputList,
} from "@components/config/util";
import { ImageConfig } from "./components/image";
import { RestartModeSelector } from "./components/restart";
import { NetworkModeSelector } from "./components/network";
import { Config } from "@components/config";
import { ResourceLink, ResourceSelector } from "@components/resources/common";
import { Link } from "react-router-dom";
import { SecretsSearch } from "@components/config/env_vars";
import { MonacoEditor } from "@components/monaco";
import {
  DefaultTerminationSignal,
  TerminationTimeout,
} from "./components/term-signal";
import { extract_registry_domain } from "@lib/utils";
import { Button } from "@ui/button";
import { X } from "lucide-react";

export const DeploymentConfig = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) => {
  const { canWrite } = usePermissions({ type: "Deployment", id });
  const config = useRead("GetDeployment", { deployment: id }).data?.config;
  const builds = useRead("ListBuilds", {}).data;
  const global_disabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const swarms_exist = useRead("ListSwarms", {}).data?.length ? true : false;
  const [update, set] = useLocalStorage<Partial<Types.DeploymentConfig>>(
    `deployment-${id}-update-v1`,
    {}
  );
  const { mutateAsync } = useWrite("UpdateDeployment");

  if (!config) return null;

  const network = update.network ?? config.network;
  const hide_ports = network === "host" || network === "none";
  const auto_update = update.auto_update ?? config.auto_update ?? false;

  const disabled = global_disabled || !canWrite;

  const curr_swarm_id = update.swarm_id ?? config.swarm_id;
  const curr_server_id = update.server_id ?? config.server_id;
  const ClearServerSwarmButton = (
    <Button
      size="icon"
      variant="secondary"
      onClick={() =>
        set((update) => ({ ...update, swarm_id: "", server_id: "" }))
      }
      disabled={disabled}
    >
      <X className="w-4 h-4" />
    </Button>
  );

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
      components={{
        "": [
          {
            label: "Swarm",
            labelHidden: true,
            hidden: !swarms_exist || !!curr_server_id,
            components: {
              swarm_id: (swarm_id, set) => {
                return (
                  <ConfigItem
                    label={
                      swarm_id ? (
                        <div className="flex gap-3 text-lg font-bold">
                          Swarm:
                          <ResourceLink type="Swarm" id={swarm_id} />
                        </div>
                      ) : (
                        "Select Swarm"
                      )
                    }
                    description="Select the Swarm to deploy on."
                  >
                    <div className="flex items-center gap-4">
                      <ResourceSelector
                        type="Swarm"
                        selected={swarm_id}
                        onSelect={(swarm_id) => set({ swarm_id })}
                        disabled={disabled}
                        align="start"
                      />
                      {ClearServerSwarmButton}
                    </div>
                  </ConfigItem>
                );
              },
            },
          },
          {
            label: "Server",
            labelHidden: true,
            hidden: !!curr_swarm_id,
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
                    <div className="flex items-center gap-4">
                      <ResourceSelector
                        type="Server"
                        selected={server_id}
                        onSelect={(server_id) => set({ server_id })}
                        disabled={disabled}
                        align="start"
                      />
                      {ClearServerSwarmButton}
                    </div>
                  </ConfigItem>
                );
              },
            },
          },
          {
            label:
              (update.image ?? config.image)?.type === "Build"
                ? "Build"
                : "Image",
            description:
              "Either pass a docker image directly, or choose a Build to deploy",
            components: {
              image: (value, set) => (
                <ImageConfig image={value} set={set} disabled={disabled} />
              ),
              image_registry_account: (account, set) => {
                const image = update.image ?? config.image;
                const provider =
                  image?.type === "Image" && image.params.image
                    ? extract_registry_domain(image.params.image)
                    : image?.type === "Build" && image.params.build_id
                      ? builds?.find((b) => b.id === image.params.build_id)
                          ?.info.image_registry_domain
                      : undefined;
                return (
                  <AccountSelectorConfig
                    id={update.server_id ?? config.server_id ?? undefined}
                    type="Server"
                    account_type="docker"
                    provider={provider ?? "docker.io"}
                    selected={account}
                    onSelect={(image_registry_account) =>
                      set({ image_registry_account })
                    }
                    disabled={disabled}
                    placeholder={
                      image?.type === "Build" ? "Same as Build" : undefined
                    }
                    description={
                      image?.type === "Build"
                        ? "Select an alternate account used to log in to the provider"
                        : undefined
                    }
                  />
                );
              },
              redeploy_on_build: (update.image?.type ?? config.image?.type) ===
                "Build" && {
                description: "Automatically redeploy when the image is built.",
              },
            },
          },
          {
            label: "Network",
            labelHidden: true,
            components: {
              network: (value, set) => (
                <NetworkModeSelector
                  swarm_id={update.swarm_id ?? config.swarm_id}
                  server_id={update.server_id ?? config.server_id}
                  selected={value}
                  onSelect={(network) => set({ network })}
                  disabled={disabled}
                />
              ),
              ports:
                !hide_ports &&
                ((ports, set) => (
                  <ConfigItem
                    label="Ports"
                    description="Configure port mappings."
                  >
                    <MonacoEditor
                      value={ports || "  # 3000:3000\n"}
                      language="key_value"
                      onValueChange={(ports) => set({ ports })}
                      readOnly={disabled}
                    />
                  </ConfigItem>
                )),
              links: (values, set) => (
                <ConfigList
                  label="Links"
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
          {
            label: "Environment",
            description: "Pass these variables to the container",
            components: {
              environment: (env, set) => (
                <div className="flex flex-col gap-4">
                  <SecretsSearch
                    server={update.server_id ?? config.server_id}
                  />
                  <MonacoEditor
                    value={env || "  # VARIABLE = value\n"}
                    onValueChange={(environment) => set({ environment })}
                    language="key_value"
                    readOnly={disabled}
                  />
                </div>
              ),
              // skip_secret_interp: true,
            },
          },
          {
            label: "Volumes",
            description: "Configure the volume bindings.",
            components: {
              volumes: (volumes, set) => (
                <MonacoEditor
                  value={volumes || "  # volume:/container/path\n"}
                  language="key_value"
                  onValueChange={(volumes) => set({ volumes })}
                  readOnly={disabled}
                />
              ),
            },
          },
          {
            label: "Restart",
            hidden: !!curr_swarm_id,
            labelHidden: true,
            components: {
              restart: (value, set) => (
                <RestartModeSelector
                  selected={value}
                  set={set}
                  disabled={disabled}
                />
              ),
            },
          },
          {
            label: "Auto Update",
            hidden: (update.image ?? config.image)?.type === "Build",
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
            },
          },
        ],
        advanced: [
          {
            label: "Command",
            labelHidden: true,
            components: {
              command: (value, set) => (
                <ConfigItem
                  label="Command"
                  boldLabel
                  description={
                    <div className="flex flex-row flex-wrap gap-2">
                      <div>Replace the CMD, or extend the ENTRYPOINT.</div>
                      <Link
                        to={
                          curr_swarm_id
                            ? "https://docs.docker.com/reference/cli/docker/service/create/#create-a-service"
                            : "https://docs.docker.com/engine/reference/run/#commands-and-arguments"
                        }
                        target="_blank"
                        className="text-primary"
                      >
                        See docker docs.
                      </Link>
                    </div>
                  }
                >
                  <MonacoEditor
                    value={value}
                    language="shell"
                    onValueChange={(command) => set({ command })}
                    readOnly={disabled}
                  />
                </ConfigItem>
              ),
            },
          },
          {
            label: "Labels",
            description: "Attach --labels to the container.",
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
                      <div>
                        Pass extra arguments to '
                        {curr_swarm_id ? "docker service create" : "docker run"}
                        '.
                      </div>
                      <Link
                        to={
                          curr_swarm_id
                            ? "https://docs.docker.com/reference/cli/docker/service/create/#options"
                            : "https://docs.docker.com/reference/cli/docker/container/run/#options"
                        }
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
                      type="Deployment"
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
            label: "Termination",
            hidden: !!curr_swarm_id,
            description:
              "Configure the signals used to 'docker stop' the container. Options are SIGTERM, SIGQUIT, SIGINT, and SIGHUP.",
            components: {
              termination_signal: (value, set) => (
                <DefaultTerminationSignal
                  arg={value}
                  set={set}
                  disabled={disabled}
                />
              ),
              termination_timeout: (value, set) => (
                <TerminationTimeout arg={value} set={set} disabled={disabled} />
              ),
              term_signal_labels: (value, set) => (
                <ConfigItem
                  label="Termination Signal Labels"
                  description="Choose between multiple signals when stopping"
                >
                  <MonacoEditor
                    value={value || DEFAULT_TERM_SIGNAL_LABELS}
                    language="key_value"
                    onValueChange={(term_signal_labels) =>
                      set({ term_signal_labels })
                    }
                    readOnly={disabled}
                  />
                </ConfigItem>
              ),
            },
          },
        ],
      }}
    />
  );
};

export const DEFAULT_TERM_SIGNAL_LABELS = `  # SIGTERM: sigterm label
  # SIGQUIT: sigquit label
  # SIGINT: sigint label
  # SIGHUP: sighup label
`;
