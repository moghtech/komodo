import { Config } from "@components/config";
import { ConfigItem, ConfigList } from "@components/config/util";
import { useLocalStorage, usePermissions, useRead, useWrite } from "@lib/hooks";
import { Types } from "komodo_client";
import { useState } from "react";
import { ResourceLink, ResourceSelector } from "../common";
import { Button } from "@ui/button";
import { MinusCircle, PlusCircle } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTrigger,
} from "@ui/dialog";
import { Card } from "@ui/card";
import { cn } from "@lib/utils";
import { Input } from "@ui/input";
import { MonacoEditor } from "@components/monaco";

export const BuilderConfig = ({ id }: { id: string }) => {
  const config = useRead("GetBuilder", { builder: id }).data?.config;
  if (config?.type === "Aws") return <AwsBuilderConfig id={id} />;
  if (config?.type === "Server") return <ServerBuilderConfig id={id} />;
  if (config?.type === "Url") return <UrlBuilderConfig id={id} />;
};

const AwsBuilderConfig = ({ id }: { id: string }) => {
  const { canWrite } = usePermissions({ type: "Builder", id });
  const config = useRead("GetBuilder", { builder: id }).data?.config
    ?.params as Types.AwsBuilderConfig;
  const global_disabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const [update, set] = useLocalStorage<Partial<Types.AwsBuilderConfig>>(
    `aws-builder-${id}-update-v1`,
    {}
  );
  const { mutateAsync } = useWrite("UpdateBuilder");
  if (!config) return null;

  const disabled = global_disabled || !canWrite;

  return (
    <Config
      disabled={disabled}
      original={config}
      update={update}
      set={set}
      onSave={async () => {
        await mutateAsync({ id, config: { type: "Aws", params: update } });
      }}
      components={{
        "": [
          {
            label: "General",
            components: {
              region: {
                description:
                  "Configure the AWS region to launch the instance in.",
                placeholder: "Input region",
              },
              instance_type: {
                description: "Choose the instance type to launch",
                placeholder: "Input instance type",
              },
              ami_id: {
                description:
                  "Create an Ami with Docker and Komodo Periphery installed.",
                placeholder: "Input Ami Id",
              },
              volume_gb: {
                description: "The size of the disk to attach to the instance.",
                placeholder: "Input size",
              },
              key_pair_name: {
                description: "Attach a key pair to the instance",
                placeholder: "Input key pair name",
              },
            },
          },
          {
            label: "Network",
            components: {
              subnet_id: {
                description: "Configure the subnet to launch the instance in.",
                placeholder: "Input subnet id",
              },
              security_group_ids: (values, set) => (
                <ConfigList
                  label="Security Group Ids"
                  description="Attach security groups to the instance."
                  field="security_group_ids"
                  values={values ?? []}
                  set={set}
                  disabled={disabled}
                  placeholder="Input Id"
                />
              ),
              assign_public_ip: {
                description:
                  "Whether to assign a public IP to the build instance.",
              },
              use_public_ip: {
                description:
                  "Whether to connect to the instance over the public IP. Otherwise, will use the internal IP.",
              },
              port: {
                description: "Configure the port to connect to Periphery on.",
                placeholder: "Input port",
              },
              use_https: {
                description: "Whether to connect to Periphery using HTTPS.",
              },
            },
          },
          {
            label: "User Data",
            description: "Run a script to setup the instance.",
            components: {
              user_data: (user_data, set) => {
                return (
                  <MonacoEditor
                    value={user_data}
                    language="shell"
                    onValueChange={(user_data) => set({ user_data })}
                    readOnly={disabled}
                  />
                );
              },
            },
          },
        ],
        additional: [
          {
            label: "Git Providers",
            boldLabel: false,
            description:
              "If you configured additional git providers / tokens in Periphery config on the builder, add them here so they will be suggested.",
            components: {
              git_providers: (providers, set) =>
                providers && (
                  <>
                    {!disabled && (
                      <Button
                        variant="secondary"
                        onClick={() =>
                          set({
                            git_providers: [
                              ...(update.git_providers ??
                                config.git_providers ??
                                []),
                              {
                                domain: "github.com",
                                https: true,
                                accounts: [],
                              },
                            ],
                          })
                        }
                        className="flex items-center gap-2 w-[200px]"
                      >
                        <PlusCircle className="w-4 h-4" />
                        Add Git Provider
                      </Button>
                    )}
                    <ProvidersConfig
                      type="git"
                      providers={providers}
                      set={set}
                      disabled={disabled}
                    />
                  </>
                ),
            },
          },
          {
            label: "Docker Registries",
            boldLabel: false,
            description:
              "If you configured additional registries / tokens in Periphery config on the builder, add them here so they will be suggested.",
            components: {
              docker_registries: (providers, set) =>
                providers && (
                  <>
                    {!disabled && (
                      <Button
                        variant="secondary"
                        onClick={() =>
                          set({
                            docker_registries: [
                              ...(update.docker_registries ??
                                config.docker_registries ??
                                []),
                              {
                                domain: "docker.io",
                                accounts: [],
                                organizations: [],
                              },
                            ],
                          })
                        }
                        className="flex items-center gap-2 w-[200px]"
                      >
                        <PlusCircle className="w-4 h-4" />
                        Add Docker Registry
                      </Button>
                    )}
                    <ProvidersConfig
                      type="docker"
                      providers={providers}
                      set={set}
                      disabled={disabled}
                    />
                  </>
                ),
            },
          },
          {
            label: "Secret Keys",
            labelHidden: true,
            components: {
              secrets: (secrets, set) => (
                <ConfigList
                  label="Secret Keys"
                  description="If you configured additional secrets in Periphery config on the builder, add them here so they will be suggested."
                  field="secrets"
                  values={secrets ?? []}
                  set={set}
                  disabled={disabled}
                  placeholder="SECRET_KEY"
                />
              ),
            },
          },
        ],
      }}
    />
  );
};

const ServerBuilderConfig = ({ id }: { id: string }) => {
  const { canWrite } = usePermissions({ type: "Builder", id });
  const config = useRead("GetBuilder", { builder: id }).data?.config;
  const [update, set] = useLocalStorage<Partial<Types.ServerBuilderConfig>>(
    `server-builder-${id}-update-v1`,
    {}
  );
  const { mutateAsync } = useWrite("UpdateBuilder");
  if (!config) return null;

  const disabled = !canWrite;

  return (
    <Config
      disabled={disabled}
      original={config.params as Types.ServerBuilderConfig}
      update={update}
      set={set}
      onSave={async () => {
        await mutateAsync({ id, config: { type: "Server", params: update } });
      }}
      components={{
        "": [
          {
            label: "Server",
            labelHidden: true,
            components: {
              server_id: (server_id, set) => {
                return (
                  <ConfigItem
                    label={
                      server_id ? (
                        <div className="flex gap-3 text-lg">
                          Server:
                          <ResourceLink type="Server" id={server_id} />
                        </div>
                      ) : (
                        "Select Server"
                      )
                    }
                    description="Select the Server to build on."
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
          },
        ],
      }}
    />
  );
};

const UrlBuilderConfig = ({ id }: { id: string }) => {
  const { canWrite } = usePermissions({ type: "Builder", id });
  const config = useRead("GetBuilder", { builder: id }).data?.config;
  const [update, set] = useLocalStorage<Partial<Types.UrlBuilderConfig>>(
    `url-builder-${id}-update-v1`,
    {}
  );
  const { mutateAsync } = useWrite("UpdateBuilder");
  if (!config) return null;

  const disabled = !canWrite;

  return (
    <Config
      disabled={disabled}
      original={config.params as Types.UrlBuilderConfig}
      update={update}
      set={set}
      onSave={async () => {
        await mutateAsync({ id, config: { type: "Url", params: update } });
      }}
      components={{
        "": [
          {
            label: "General",
            labelHidden: true,
            components: {
              address: {
                description: "The address of the Periphery agent",
                placeholder: "https://periphery:8120",
              },
              passkey: {
                description:
                  "Use a custom passkey to authenticate with Periphery",
                placeholder: "Custom passkey",
              },
            },
          },
        ],
      }}
    />
  );
};

const ProvidersConfig = (params: {
  type: "git" | "docker";
  providers: Types.GitProvider[] | Types.DockerRegistry[];
  set: (input: Partial<Types.AwsBuilderConfig>) => void;
  disabled: boolean;
}) => {
  const arr_field =
    params.type === "git" ? "git_providers" : "docker_registries";
  if (!params.providers.length) return null;
  return (
    <div className="w-full flex">
      <div className="flex flex-col gap-4 w-full max-w-[400px]">
        {params.providers?.map((_, index) => (
          <div key={index} className="flex items-center justify-between gap-4">
            <ProviderDialog {...params} index={index} />
            {!params.disabled && (
              <Button
                variant="secondary"
                onClick={() =>
                  params.set({
                    [arr_field]: params.providers.filter((_, i) => i !== index),
                  })
                }
              >
                <MinusCircle className="w-4 h-4" />
              </Button>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

const ProviderDialog = ({
  type,
  providers,
  set,
  disabled,
  index,
}: {
  type: "git" | "docker";
  providers: Types.GitProvider[] | Types.DockerRegistry[];
  index: number;
  set: (input: Partial<Types.AwsBuilderConfig>) => void;
  disabled: boolean;
}) => {
  const [open, setOpen] = useState(false);
  const provider = providers[index];
  const arr_field = type === "git" ? "git_providers" : "docker_registries";
  const example_domain = type === "git" ? "github.com" : "docker.io";
  const update_domain = (domain: string) =>
    set({
      [arr_field]: providers.map((provider, i) =>
        i === index ? { ...provider, domain } : provider
      ),
    });
  const add_account = () =>
    set({
      [arr_field]: providers.map(
        (provider: Types.GitProvider | Types.DockerRegistry, i) =>
          i === index
            ? {
                ...provider,
                accounts: [...(provider.accounts ?? []), { username: "" }],
              }
            : provider
      ) as Types.GitProvider[] | Types.DockerRegistry[],
    });
  const update_username = (username: string, account_index: number) =>
    set({
      [arr_field]: providers.map(
        (provider: Types.GitProvider | Types.DockerRegistry, provider_index) =>
          provider_index === index
            ? {
                ...provider,
                accounts: provider.accounts?.map((account, i) =>
                  account_index === i ? { username } : account
                ),
              }
            : provider
      ) as Types.GitProvider[] | Types.DockerRegistry[],
    });
  const remove_account = (account_index) =>
    set({
      [arr_field]: providers.map(
        (provider: Types.GitProvider | Types.DockerRegistry, provider_index) =>
          provider_index === index
            ? {
                ...provider,
                accounts: provider.accounts?.filter(
                  (_, i) => account_index !== i
                ),
              }
            : provider
      ) as Types.GitProvider[] | Types.DockerRegistry[],
    });
  const add_organization = () =>
    set({
      [arr_field]: providers.map((provider: Types.DockerRegistry, i) =>
        i === index
          ? {
              ...provider,
              organizations: [...(provider.organizations ?? []), ""],
            }
          : provider
      ) as Types.DockerRegistry[],
    });
  const update_organization = (name: string, organization_index: number) =>
    set({
      [arr_field]: providers.map(
        (provider: Types.DockerRegistry, provider_index) =>
          provider_index === index
            ? {
                ...provider,
                organizations: provider.organizations?.map((organization, i) =>
                  organization_index === i ? name : organization
                ),
              }
            : provider
      ) as Types.GitProvider[] | Types.DockerRegistry[],
    });
  const remove_organization = (organization_index) =>
    set({
      [arr_field]: providers.map(
        (provider: Types.DockerRegistry, provider_index) =>
          provider_index === index
            ? {
                ...provider,
                organizations: provider.organizations?.filter(
                  (_, i) => organization_index !== i
                ),
              }
            : provider
      ) as Types.DockerRegistry[],
    });
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Card className="px-3 py-2 hover:bg-accent/50 transition-colors cursor-pointer w-full">
          <div
            className={cn(
              "flex gap-2 text-sm text-nowrap overflow-hidden overflow-ellipsis"
            )}
          >
            <div className="flex gap-2">{provider.domain}</div>
            <div className="flex gap-2">
              <div className="text-muted-foreground">accounts:</div>{" "}
              {provider.accounts?.length || 0}
            </div>
            {(provider as Types.DockerRegistry).organizations !== undefined && (
              <div className="flex gap-2">
                <div className="text-muted-foreground">organizations:</div>{" "}
                {(provider as Types.DockerRegistry).organizations?.length || 0}
              </div>
            )}
          </div>
        </Card>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          {type === "git" ? "Git Provider" : "Docker Registry"}
        </DialogHeader>
        <div className="flex flex-col gap-4">
          {/* Domain */}
          <div className="flex items-center justify-between w-fill">
            <div className="text-nowrap">Domain</div>
            <Input
              value={provider.domain}
              onChange={(e) => update_domain(e.target.value)}
              disabled={disabled}
              className="w-[300px]"
              placeholder={example_domain}
            />
          </div>

          {/* Accounts */}
          <div className="flex flex-col gap-2 w-fill">
            <div className="flex items-center justify-between w-fill">
              <div className="text-nowrap">Available Accounts</div>
              <Button variant="secondary" onClick={add_account}>
                Add
              </Button>
            </div>
            <div className="flex flex-col gap-2">
              {provider.accounts?.map((account, account_index) => {
                return (
                  <div
                    key={account_index}
                    className="flex gap-2 items-center justify-end"
                  >
                    <Input
                      placeholder="Account Username"
                      value={account.username}
                      onChange={(e) =>
                        update_username(e.target.value, account_index)
                      }
                    />
                    {!disabled && (
                      <Button
                        variant="secondary"
                        onClick={() => remove_account(account_index)}
                      >
                        <MinusCircle className="w-4 h-4" />
                      </Button>
                    )}
                  </div>
                );
              })}
            </div>
          </div>

          {/* Organizations */}
          {type === "docker" && (
            <div className="flex flex-col gap-2 w-fill">
              <div className="flex items-center justify-between w-fill">
                <div className="text-nowrap">Available Organizations</div>
                <Button variant="secondary" onClick={add_organization}>
                  Add
                </Button>
              </div>
              <div className="flex flex-col gap-2">
                {(provider as Types.DockerRegistry).organizations?.map(
                  (organization, organization_index) => {
                    return (
                      <div
                        key={organization_index}
                        className="flex gap-2 items-center justify-end"
                      >
                        <Input
                          value={organization}
                          onChange={(e) =>
                            update_organization(
                              e.target.value,
                              organization_index
                            )
                          }
                          placeholder="Organization Name"
                        />
                        {!disabled && (
                          <Button
                            variant="secondary"
                            onClick={() =>
                              remove_organization(organization_index)
                            }
                          >
                            <MinusCircle className="w-4 h-4" />
                          </Button>
                        )}
                      </div>
                    );
                  }
                )}
              </div>
            </div>
          )}
        </div>
        <DialogFooter>
          <Button onClick={() => setOpen(false)}>Confirm</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
