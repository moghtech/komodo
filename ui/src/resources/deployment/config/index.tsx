import { usePermissions, useRead, useWrite } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import Config from "@/ui/config";
import { ConfigItem, ConfigList, ConfigSwitch } from "@/ui/config/item";
import { ActionIcon, Group, Stack, Text } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import { ResourceLink } from "@/resources/common";
import ResourceSelector from "@/resources/selector";
import { AccountSelectorConfig } from "@/components/config/account-selector";
import { extractRegistryDomain } from "@/lib/utils";
import DeploymentImageConfig from "./image";
import { MonacoEditor } from "@/components/monaco";
import DeploymentNetworkSelector from "./network";
import SecretsSearch from "@/components/config/secrets-search";
import DeploymentRestartSelector from "./restart";
import { Link } from "react-router-dom";
import AddExtraArg from "@/components/config/add-extra-arg";
import InputList from "@/ui/input-list";
import { TerminationSignal, TerminationTimeout } from "./termination";

export default function DeploymentConfig({ id }: { id: string }) {
  const { canWrite } = usePermissions({ type: "Deployment", id });
  const config = useRead("GetDeployment", { deployment: id }).data?.config;
  const builds = useRead("ListBuilds", {}).data;
  const globalDisabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const swarmsExist = useRead("ListSwarms", {}).data?.length ? true : false;
  const [update, setUpdate] = useLocalStorage<Partial<Types.DeploymentConfig>>({
    key: `deployment-${id}-update-v1`,
    defaultValue: {},
  });
  const { mutateAsync } = useWrite("UpdateDeployment");

  if (!config) return null;

  const network = update.network ?? config.network;
  const hidePorts = network === "host" || network === "none";
  const autoUpdate = update.auto_update ?? config.auto_update ?? false;

  const disabled = globalDisabled || !canWrite;

  const currSwarmId = update.swarm_id ?? config.swarm_id;
  const currServerId = update.server_id ?? config.server_id;
  const ClearServerSwarmButton = (
    <ActionIcon
      color="red"
      onClick={() =>
        setUpdate((update) => ({ ...update, swarm_id: "", server_id: "" }))
      }
      disabled={disabled}
    >
      <ICONS.Clear size="1rem" />
    </ActionIcon>
  );

  return (
    <Config
      disabled={disabled}
      original={config}
      update={update}
      setUpdate={setUpdate}
      onSave={() => mutateAsync({ id, config: update })}
      groups={{
        "": [
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
                        <Group>
                          Swarm:
                          <ResourceLink type="Swarm" id={swarm_id} />
                        </Group>
                      ) : (
                        "Select Swarm"
                      )
                    }
                    description="Select the Swarm to deploy on."
                  >
                    <Group>
                      <ResourceSelector
                        type="Swarm"
                        selected={swarm_id}
                        onSelect={(swarm_id) => set({ swarm_id })}
                        disabled={disabled}
                      />
                      {ClearServerSwarmButton}
                    </Group>
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
                        <Group>
                          Server:
                          <ResourceLink type="Server" id={server_id} />
                        </Group>
                      ) : (
                        "Select Server"
                      )
                    }
                    description="Select the Server to deploy on."
                  >
                    <Group>
                      <ResourceSelector
                        type="Server"
                        selected={server_id}
                        onSelect={(server_id) => set({ server_id })}
                        disabled={disabled}
                      />
                      {ClearServerSwarmButton}
                    </Group>
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
            fields: {
              image: (value, set) => (
                <DeploymentImageConfig
                  image={value}
                  setUpdate={set}
                  disabled={disabled}
                />
              ),
              image_registry_account: (account, set) => {
                const image = update.image ?? config.image;
                const provider =
                  image?.type === "Image" && image.params.image
                    ? extractRegistryDomain(image.params.image)
                    : image?.type === "Build" && image.params.build_id
                      ? builds?.find((b) => b.id === image.params.build_id)
                          ?.info.image_registry_domain
                      : undefined;
                return (
                  <AccountSelectorConfig
                    id={update.server_id ?? config.server_id ?? undefined}
                    type="Server"
                    accountType="docker"
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
            fields: {
              network: (value, set) => (
                <DeploymentNetworkSelector
                  swarmId={update.swarm_id ?? config.swarm_id}
                  serverId={update.server_id ?? config.server_id}
                  selected={value}
                  onSelect={(network) => set({ network })}
                  disabled={disabled}
                />
              ),
              ports:
                !hidePorts &&
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
            fields: {
              environment: (env, set) => (
                <Stack>
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
              // skip_secret_interp: true,
            },
          },
          {
            label: "Volumes",
            description: "Configure the volume bindings.",
            fields: {
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
            hidden: !!currSwarmId,
            labelHidden: true,
            fields: {
              restart: (value, set) => (
                <DeploymentRestartSelector
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
            fields: {
              poll_for_updates: (poll, set) => {
                return (
                  <ConfigSwitch
                    label="Poll for Updates"
                    description="Check for updates to the image on an interval."
                    value={autoUpdate || poll}
                    onChange={(poll_for_updates) => set({ poll_for_updates })}
                    disabled={disabled || autoUpdate}
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
            fields: {
              command: (value, set) => (
                <ConfigItem
                  label="Command"
                  boldLabel
                  description={
                    <Group>
                      <Text>Replace the CMD, or extend the ENTRYPOINT.</Text>
                      <Link
                        to={
                          currSwarmId
                            ? "https://docs.docker.com/reference/cli/docker/service/create/#create-a-service"
                            : "https://docs.docker.com/engine/reference/run/#commands-and-arguments"
                        }
                        target="_blank"
                      >
                        See docker docs.
                      </Link>
                    </Group>
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
          {
            label: "Extra Args",
            labelHidden: true,
            fields: {
              extra_args: (value, set) => (
                <ConfigItem
                  label="Extra Args"
                  boldLabel
                  description={
                    <div className="flex flex-row flex-wrap gap-2">
                      <div>
                        Pass extra arguments to '
                        {currSwarmId ? "docker service create" : "docker run"}
                        '.
                      </div>
                      <Link
                        to={
                          currSwarmId
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
                    <AddExtraArg
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
            hidden: !!currSwarmId,
            description:
              "Configure the signals used to 'docker stop' the container. Options are SIGTERM, SIGQUIT, SIGINT, and SIGHUP.",
            fields: {
              termination_signal: (value, set) => (
                <TerminationSignal arg={value} set={set} disabled={disabled} />
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
}

const DEFAULT_TERM_SIGNAL_LABELS = `  # SIGTERM: sigterm label
  # SIGQUIT: sigquit label
  # SIGINT: sigint label
  # SIGHUP: sighup label
`;
