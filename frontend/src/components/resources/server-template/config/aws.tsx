import { Config } from "@components/config";
import { ConfigList } from "@components/config/util";
import { useLocalStorage, useRead, useWrite } from "@lib/hooks";
import { cn } from "@lib/utils";
import { Types } from "komodo_client";
import { Button } from "@ui/button";
import { Card } from "@ui/card";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTrigger,
} from "@ui/dialog";
import { Input } from "@ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { MinusCircle, PlusCircle } from "lucide-react";
import { useState } from "react";
import { MonacoEditor } from "@components/monaco";

export const AwsServerTemplateConfig = ({
  id,
  disabled,
}: {
  id: string;
  disabled: boolean;
}) => {
  const config = useRead("GetServerTemplate", { server_template: id }).data
    ?.config?.params as Types.AwsServerTemplateConfig;
  const [update, set] = useLocalStorage<Partial<Types.AwsServerTemplateConfig>>(
    `aws-template-${id}-update-v1`,
    {}
  );
  const { mutateAsync } = useWrite("UpdateServerTemplate");
  if (!config) return null;

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
              use_public_ip:
                update.assign_public_ip ?? config.assign_public_ip
                  ? {
                      description:
                        "Whether to connect to the instance over the public IP. Otherwise, will use the internal IP.",
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
                                (update.volumes ?? config.volumes).length
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
                    <div className="w-full flex">
                      <div className="flex flex-col gap-4 w-full max-w-[400px]">
                        {volumes.map((_, index) => (
                          <div
                            key={index}
                            className="flex items-center justify-between gap-4"
                          >
                            <AwsVolumeDialog
                              volumes={volumes}
                              index={index}
                              set={set}
                              disabled={disabled}
                            />
                            {!disabled && (
                              <Button
                                variant="secondary"
                                disabled={disabled}
                                onClick={() =>
                                  set({
                                    volumes: volumes.filter(
                                      (_, i) => i !== index
                                    ),
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
                  </>
                );
              },
            },
          },
          {
            label: "User Data",
            description: "Run a script to setup the instance",
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
      }}
    />
  );
};

const AwsVolumeDialog = ({
  volumes,
  index,
  set,
  disabled,
}: {
  volumes: Types.AwsVolume[];
  index: number;
  set: (value: Partial<Types.AwsServerTemplateConfig>) => void;
  disabled?: boolean;
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
              <div className="text-muted-foreground">device:</div>{" "}
              {volume.device_name}
            </div>
            <div className="flex gap-2">
              <div className="text-muted-foreground">size:</div>{" "}
              {volume.size_gb} GB
            </div>
            <div className="flex gap-2">
              <div className="text-muted-foreground">type:</div>{" "}
              {volume.volume_type}
            </div>
          </div>
        </Card>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>AWS Volume</DialogHeader>
        <div className="flex flex-col gap-4">
          {/* Device Name */}
          <div className="flex items-center justify-between w-fill">
            <div className="text-nowrap">Device Name</div>
            <Input
              value={volume.device_name}
              onChange={(e) =>
                set({
                  volumes: volumes.map((volume, i) =>
                    i === index
                      ? { ...volume, device_name: e.target.value }
                      : volume
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
            <div className="text-nowrap">Volume Type</div>
            <Select
              value={volume.volume_type}
              onValueChange={(value) =>
                set({
                  volumes: volumes.map((volume, i) =>
                    i === index
                      ? { ...volume, volume_type: value as Types.AwsVolumeType }
                      : volume
                  ),
                })
              }
            >
              <SelectTrigger className="w-[300px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {Object.entries(Types.AwsVolumeType).map(([name, item]) => (
                  <SelectItem key={name} value={item}>
                    {name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* IOPS */}
          <div className="flex items-center justify-between w-fill">
            <div className="text-nowrap">Iops</div>
            <Input
              type="number"
              placeholder="Set Custom Iops"
              value={volume.iops ? volume.iops : undefined}
              onChange={(e) =>
                set({
                  volumes: volumes.map((volume, i) =>
                    i === index
                      ? { ...volume, iops: Number(e.target.value || 0) }
                      : volume
                  ),
                })
              }
              disabled={disabled}
              className="w-[300px]"
            />
          </div>

          {/* Throughput */}
          <div className="flex items-center justify-between w-fill">
            <div className="text-nowrap">Throughput</div>
            <Input
              type="number"
              placeholder="Set Custom Throughput"
              value={volume.throughput ? volume.throughput : undefined}
              onChange={(e) =>
                set({
                  volumes: volumes.map((volume, i) =>
                    i === index
                      ? { ...volume, throughput: Number(e.target.value || 0) }
                      : volume
                  ),
                })
              }
              disabled={disabled}
              className="w-[300px]"
            />
          </div>
        </div>
        <DialogFooter>
          <Button onClick={() => setOpen(false)}>Confirm</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

const device_letters = ["b", "c", "d", "e", "f"];

const newVolume: (index: number) => Types.AwsVolume = (index) => {
  const device_name =
    index === 0 ? "/dev/sda1" : `/dev/sd${device_letters[index - 1]}`;
  return {
    device_name,
    size_gb: 20,
    volume_type: Types.AwsVolumeType.Gp2,
    iops: 0,
    throughput: 0,
  };
};
