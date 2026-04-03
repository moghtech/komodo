import { usePermissions, useRead, useWrite } from "@/lib/hooks";
import { useFullServer } from ".";
import { ReactNode, useEffect, useState } from "react";
import { Types } from "komodo_client";
import { useLocalStorage } from "@mantine/hooks";
import Config from "@/ui/config";
import { ConfigInput, ConfigList } from "@/ui/config/item";
import ConfirmButton from "@/ui/confirm-button";
import { ICONS } from "@/lib/icons";
import { Group } from "@mantine/core";
import { useIsServerAvailable } from "./hooks";
import ConfigMaintenanceWindows from "@/components/maintenance-windows";

export default function ServerConfig({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const globalDisabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const { canWrite } = usePermissions({ type: "Server", id });
  const isAvailable = useIsServerAvailable(id);

  const server = useFullServer(id);
  const config = server?.config;
  const [publicKey, setPublicKey] = useState("");

  useEffect(() => {
    if (server?.info?.public_key) {
      setPublicKey(server.info.public_key);
    }
  }, [server?.info?.public_key]);
  const [update, setUpdate] = useLocalStorage<Partial<Types.ServerConfig>>({
    key: `server-${id}-update-v1`,
    defaultValue: {},
  });

  const { mutateAsync } = useWrite("UpdateServer");
  const { mutate: updatePublicKey, isPending: updatePublicPending } = useWrite(
    "UpdateServerPublicKey",
  );
  const { mutate: rotate, isPending: rotatePending } =
    useWrite("RotateServerKeys");

  if (!config) return null;

  const disabled = globalDisabled || !canWrite;
  const address = update.address ?? config.address;
  const tlsAddress = !!address && !address.startsWith("ws://");

  return (
    <Config
      titleOther={titleOther}
      disabled={disabled}
      original={config}
      update={update}
      setUpdate={setUpdate}
      onSave={() => mutateAsync({ id, config: update })}
      groups={{
        "": [
          {
            label: "Enabled",
            labelHidden: true,
            fields: {
              enabled: {
                description:
                  "Whether to attempt to connect to this host / send alerts if offline. Disabling will also convert all attached resource's state to 'Unknown'.",
              },
            },
          },
          {
            label: "Auth",
            labelHidden: true,
            fields: {
              enabled: () => (
                <ConfigInput
                  label="Periphery Public Key"
                  description="If provided, the associated private key must be set as Periphery 'private_key'. For Periphery -> Core connection, either this or using 'periphery_public_key' in Core config is required for Periphery to be able to connect."
                  placeholder="custom-public-key"
                  value={publicKey}
                  onValueChange={(publicKey) => setPublicKey(publicKey)}
                  inputRight={
                    !disabled && (
                      <Group>
                        <ConfirmButton
                          icon={<ICONS.Save size="1rem" />}
                          maw="120px"
                          onClick={() =>
                            updatePublicKey({
                              server: id,
                              public_key: publicKey,
                            })
                          }
                          loading={updatePublicPending}
                          disabled={publicKey === server?.info?.public_key}
                        >
                          Save
                        </ConfirmButton>
                        <ConfirmButton
                          icon={<ICONS.RotateKey size="1rem" />}
                          maw="120px"
                          onClick={() => rotate({ server: id })}
                          loading={rotatePending}
                          disabled={!isAvailable}
                        >
                          Rotate
                        </ConfirmButton>
                      </Group>
                    )
                  }
                  disabled={disabled}
                />
              ),
              auto_rotate_keys: {
                description:
                  "Include in key rotation with 'RotateAllServerKeys'.",
              },
            },
          },
          {
            label: "Address",
            labelHidden: true,
            fields: {
              address: {
                description:
                  "For Core -> Periphery connection mode, specify address of periphery in your network.",
                placeholder: "12.34.56.78:8120",
              },
              insecure_tls: {
                hidden: !tlsAddress,
                description: "Skip Periphery TLS certificate validation.",
              },
              external_address: {
                description:
                  "Optional. The address of the server used in container links, if different than the Address.",
                placeholder: "my.server.int",
              },
              region: {
                description:
                  "Optional. Attach a region to the server for visual grouping.",
                placeholder: "Configure Region",
              },
            },
          },
          {
            label: "Disks",
            labelHidden: true,
            fields: {
              ignore_mounts: (values, set) => (
                <ConfigList
                  description="If undesired disk mount points are coming through in server stats, filter them out here."
                  label="Ignore Disks"
                  field="ignore_mounts"
                  values={values ?? []}
                  set={set}
                  disabled={disabled}
                  placeholder="/path/to/disk"
                />
              ),
            },
          },
          {
            label: "Monitoring",
            labelHidden: true,
            fields: {
              stats_monitoring: {
                label: "System Stats Monitoring",
                description:
                  "Whether to store historical CPU, RAM, and disk usage.",
              },
            },
          },
          {
            label: "Pruning",
            labelHidden: true,
            fields: {
              auto_prune: {
                label: "Auto Prune Images",
                description:
                  "Whether to prune unused images every day at UTC 00:00",
              },
            },
          },
        ],
        alerts: [
          {
            label: "Unreachable",
            labelHidden: true,
            fields: {
              send_unreachable_alerts: {
                description:
                  "Send an alert if the Periphery agent cannot be reached.",
              },
            },
          },
          {
            label: "Version",
            labelHidden: true,
            fields: {
              send_version_mismatch_alerts: {
                label: "Send Version Mismatch Alerts",
                description:
                  "Send an alert if the Periphery version differs from the Core version.",
              },
            },
          },
          {
            label: "CPU",
            labelHidden: true,
            fields: {
              send_cpu_alerts: {
                label: "Send CPU Alerts",
                description:
                  "Send an alert if the CPU usage is above the configured thresholds.",
              },
              cpu_warning: {
                description:
                  "Send a 'Warning' alert if the CPU usage in % is above these thresholds",
              },
              cpu_critical: {
                description:
                  "Send a 'Critical' alert if the CPU usage in % is above these thresholds",
              },
            },
          },
          {
            label: "Memory",
            labelHidden: true,
            fields: {
              send_mem_alerts: {
                label: "Send Memory Alerts",
                description:
                  "Send an alert if the memory usage is above the configured thresholds.",
              },
              mem_warning: {
                label: "Memory Warning",
                description:
                  "Send a 'Warning' alert if the memory usage in % is above these thresholds",
              },
              mem_critical: {
                label: "Memory Critical",
                description:
                  "Send a 'Critical' alert if the memory usage in % is above these thresholds",
              },
            },
          },
          {
            label: "Disk",
            labelHidden: true,
            fields: {
              send_disk_alerts: {
                description:
                  "Send an alert if the Disk Usage (for any mounted disk) is above the configured thresholds.",
              },
              disk_warning: {
                description:
                  "Send a 'Warning' alert if the disk usage in % is above these thresholds",
              },
              disk_critical: {
                description:
                  "Send a 'Critical' alert if the disk usage in % is above these thresholds",
              },
            },
          },
          {
            label: "Maintenance",
            description: (
              <>
                Configure maintenance windows to temporarily disable alerts
                during scheduled maintenance periods. When a maintenance window
                is active, alerts from this server will be suppressed.
              </>
            ),
            fields: {
              maintenance_windows: (values, set) => {
                return (
                  <ConfigMaintenanceWindows
                    windows={values ?? []}
                    onUpdate={(maintenance_windows) =>
                      set({ maintenance_windows })
                    }
                    disabled={disabled}
                  />
                );
              },
            },
          },
        ],
      }}
    />
  );
}
