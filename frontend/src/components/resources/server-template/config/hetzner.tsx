import { Config } from "@components/config";
import { ConfigItem, ConfigList } from "@components/config/util";
import { useLocalStorage, useRead, useWrite } from "@lib/hooks";
import { cn, filterBySplit } from "@lib/utils";
import { Types } from "komodo_client";
import { Button } from "@ui/button";
import { Card } from "@ui/card";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@ui/command";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTrigger,
} from "@ui/dialog";
import { Input } from "@ui/input";
import { Popover, PopoverContent, PopoverTrigger } from "@ui/popover";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { ChevronsUpDown, MinusCircle, PlusCircle, SearchX } from "lucide-react";
import { useState } from "react";
import { MonacoEditor } from "@components/monaco";

export const HetznerServerTemplateConfig = ({
  id,
  disabled,
}: {
  id: string;
  disabled: boolean;
}) => {
  const config = useRead("GetServerTemplate", { server_template: id }).data
    ?.config?.params as Types.HetznerServerTemplateConfig;
  const [update, set] = useLocalStorage<
    Partial<Types.HetznerServerTemplateConfig>
  >(`hetzner-template-${id}-update-v1`, {});
  const { mutateAsync } = useWrite("UpdateServerTemplate");
  if (!config) return null;

  return (
    <Config
      disabled={disabled}
      original={config}
      update={update}
      set={set}
      onSave={async () => {
        await mutateAsync({ id, config: { type: "Hetzner", params: update } });
      }}
      components={{
        "": [
          {
            label: "General",
            components: {
              datacenter: (datacenter, set) => (
                <DatacenterSelector
                  selected={datacenter}
                  set={set}
                  disabled={disabled}
                />
              ),
              server_type: (server_type, set) => (
                <ServerTypeSelector
                  datacenter={
                    update.datacenter ??
                    config.datacenter ??
                    Types.HetznerDatacenter.Nuremberg1Dc3
                  }
                  selected={server_type}
                  set={set}
                  disabled={disabled}
                />
              ),
              image: {
                description:
                  "The hetzner VM image name (default: ubuntu-24.04)",
                placeholder: "Input image name",
              },
            },
          },
          {
            label: "Network",
            components: {
              private_network_ids: (values, set) => (
                <ConfigList
                  label="Private Network Ids"
                  description="Attach the VM to private networks."
                  field="private_network_ids"
                  values={values?.map((val) => val.toString()) ?? []}
                  set={({ private_network_ids }) =>
                    set({
                      private_network_ids: (
                        private_network_ids as string[]
                      ).map((id) => Number(id)),
                    })
                  }
                  disabled={disabled}
                  placeholder="Network Id"
                />
              ),
              firewall_ids: (values, set) => (
                <ConfigList
                  label="Firewall Ids"
                  description="Attach firewall rules to the VM."
                  field="firewall_ids"
                  values={values?.map((val) => val.toString()) ?? []}
                  set={({ firewall_ids }) =>
                    set({
                      firewall_ids: (firewall_ids as string[]).map((id) =>
                        Number(id)
                      ),
                    })
                  }
                  disabled={disabled}
                  placeholder="Firewall Id"
                />
              ),
              enable_public_ipv4: {
                description:
                  "Whether to assign a public IPv4 to the build instance.",
              },
              enable_public_ipv6: {
                description:
                  "Whether to assign a public IPv6 to the build instance.",
              },
              use_public_ip:
                update.enable_public_ipv4 ?? config.enable_public_ipv4
                  ? {
                      description:
                        "Whether to connect to the instance over the public IPv4. Otherwise, will use the internal IP.",
                    }
                  : false,
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
            label: "Volumes",
            components: {
              volumes: (volumes, set) => {
                return (
                  <>
                    {!disabled && (
                      <Button
                        variant="secondary"
                        onClick={() =>
                          set({
                            volumes: [
                              ...(update.volumes ?? config.volumes ?? []),
                              newVolume(
                                (update.volumes ?? config.volumes)?.length ?? 0
                              ),
                            ],
                          })
                        }
                        className="flex items-center gap-2 w-[200px]"
                      >
                        <PlusCircle className="w-4 h-4" />
                        Add Volume
                      </Button>
                    )}
                    <HetznerVolumesConfig
                      volumes={volumes ?? []}
                      set={set}
                      disabled={disabled}
                    />
                  </>
                );
              },
            },
          },
          {
            label: "SSH Keys",
            labelHidden: true,
            components: {
              ssh_keys: (values, set) => (
                <ConfigList
                  label="SSH Keys"
                  boldLabel
                  description="Attach SSH keys to the VM. Accepts the key id or name (found on Hetzner)."
                  field="ssh_keys"
                  values={values?.map((val) => val.toString()) ?? []}
                  set={set}
                  disabled={disabled}
                  placeholder="SSH Key ID or Name"
                />
              ),
            },
          },
          {
            label: "User Data",
            description: "Use cloud-init to setup the instance.",
            components: {
              user_data: (user_data, set) => {
                return (
                  <MonacoEditor
                    value={user_data}
                    language="yaml"
                    onValueChange={(user_data) => set({ user_data })}
                    readOnly={disabled}
                  />
                );
              },
            },
          },
        ],
      }}
    />
  );
};

const DatacenterSelector = ({
  disabled,
  selected,
  set,
}: {
  disabled: boolean;
  selected: Types.HetznerDatacenter | undefined;
  set: (value: Partial<Types.HetznerServerTemplateConfig>) => void;
}) => {
  return (
    <ConfigItem label="Datacenter">
      <Select
        value={selected}
        onValueChange={(value) =>
          set({ datacenter: value as Types.HetznerDatacenter })
        }
        disabled={disabled}
      >
        <SelectTrigger
          className="w-full lg:w-[200px] max-w-[50%]"
          disabled={disabled}
        >
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {Object.values(Types.HetznerDatacenter).map((datacenter) => (
            <SelectItem key={datacenter} value={datacenter}>
              {datacenter}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </ConfigItem>
  );
};

const ServerTypeSelector = ({
  datacenter,
  disabled,
  selected,
  set,
}: {
  datacenter: Types.HetznerDatacenter;
  disabled: boolean;
  selected: Types.HetznerServerType | undefined;
  set: (value: Partial<Types.HetznerServerTemplateConfig>) => void;
}) => {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  // The US based datacenters only have Amd servers
  const filter =
    datacenter === Types.HetznerDatacenter.HillsboroDc1 ||
    datacenter === Types.HetznerDatacenter.AshburnDc1 ||
    datacenter === Types.HetznerDatacenter.SingaporeDc1
      ? (st: string) => st.includes("Amd")
      : () => true;
  const server_types = Object.values(Types.HetznerServerType).filter(filter);
  const filtered = filterBySplit(server_types, search, (item) => item);
  return (
    <ConfigItem label="Server Type">
      <div>
        <Popover open={open} onOpenChange={setOpen}>
          <PopoverTrigger asChild>
            <Button
              variant="secondary"
              className="flex gap-2 w-fit"
              disabled={disabled}
            >
              {selected ?? "Select Server Type"}
              <ChevronsUpDown className="w-3 h-3" />
            </Button>
          </PopoverTrigger>
          <PopoverContent className="w-[300px] max-h-[200px] p-0" align="end">
            <Command shouldFilter={false}>
              <CommandInput
                placeholder="Search Server Types"
                className="h-9"
                value={search}
                onValueChange={setSearch}
              />
              <CommandList>
                <CommandEmpty className="flex justify-evenly items-center">
                  No Server Types Found
                  <SearchX className="w-3 h-3" />
                </CommandEmpty>

                <CommandGroup>
                  {filtered.map((server_type) => (
                    <CommandItem
                      key={server_type}
                      onSelect={() => {
                        set({ server_type });
                        setOpen(false);
                      }}
                      className="flex items-center justify-between cursor-pointer"
                    >
                      <div className="p-1">{server_type}</div>
                    </CommandItem>
                  ))}
                </CommandGroup>
              </CommandList>
            </Command>
          </PopoverContent>
        </Popover>
      </div>
    </ConfigItem>
  );
};

const HetznerVolumesConfig = (params: {
  volumes: Types.HetznerVolumeSpecs[];
  set: (value: Partial<Types.HetznerServerTemplateConfig>) => void;
  disabled: boolean;
}) => {
  return (
    <div className="w-full flex">
      <div className="flex flex-col gap-4 w-full max-w-[400px]">
        {params.volumes?.map((_, index) => (
          <div key={index} className="flex items-center justify-between gap-4">
            <HetznerVolumeDialog {...params} index={index} />
            {!params.disabled && (
              <Button
                variant="secondary"
                onClick={() =>
                  params.set({
                    volumes: params.volumes.filter((_, i) => i !== index),
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

const HetznerVolumeDialog = ({
  volumes,
  index,
  set,
  disabled,
}: {
  volumes: Types.HetznerVolumeSpecs[];
  index: number;
  set: (value: Partial<Types.HetznerServerTemplateConfig>) => void;
  disabled: boolean;
}) => {
  const volume = volumes[index];
  const [open, setOpen] = useState(false);
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Card className="px-3 py-2 hover:bg-accent/50 transition-colors cursor-pointer w-full">
          <div
            className={cn(
              "flex gap-2 text-sm text-nowrap overflow-hidden overflow-ellipsis"
            )}
          >
            <div className="flex gap-2">
              <div className="text-muted-foreground">device:</div> {volume.name}
            </div>
            <div className="flex gap-2">
              <div className="text-muted-foreground">size:</div>{" "}
              {volume.size_gb} GB
            </div>
            <div className="flex gap-2">
              <div className="text-muted-foreground">type:</div> {volume.format}
            </div>
          </div>
        </Card>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>Hetzner Volume</DialogHeader>
        <div className="flex flex-col gap-4">
          {/* Volume Name */}
          <div className="flex items-center justify-between w-fill">
            <div className="text-nowrap">Volume Name</div>
            <Input
              value={volume.name}
              onChange={(e) =>
                set({
                  volumes: volumes.map((volume, i) =>
                    i === index ? { ...volume, name: e.target.value } : volume
                  ),
                })
              }
              disabled={disabled}
              className="w-[300px]"
            />
          </div>

          {/* Size GB */}
          <div className="flex items-center justify-between w-fill">
            <div className="text-nowrap">Size (GB)</div>
            <Input
              type="number"
              value={volume.size_gb}
              onChange={(e) =>
                set({
                  volumes: volumes.map((volume, i) =>
                    i === index
                      ? { ...volume, size_gb: Number(e.target.value || 0) }
                      : volume
                  ),
                })
              }
              disabled={disabled}
              className="w-[300px]"
            />
          </div>

          {/* Volume Type */}
          <div className="flex items-center justify-between w-fill">
            <div className="text-nowrap">Volume Format</div>
            <Select
              value={volume.format}
              onValueChange={(value) =>
                set({
                  volumes: volumes.map((volume, i) =>
                    i === index
                      ? {
                          ...volume,
                          format: value as Types.HetznerVolumeFormat,
                        }
                      : volume
                  ),
                })
              }
            >
              <SelectTrigger className="w-[300px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {Object.values(Types.HetznerVolumeFormat).map((format) => (
                  <SelectItem key={format} value={format}>
                    {format}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
        <DialogFooter>
          <Button onClick={() => setOpen(false)}>Confirm</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

const newVolume: (index: number) => Types.HetznerVolumeSpecs = (index) => {
  return {
    name: `volume-${index}`,
    size_gb: 20,
    format: Types.HetznerVolumeFormat.Xfs,
    labels: {},
  };
};
