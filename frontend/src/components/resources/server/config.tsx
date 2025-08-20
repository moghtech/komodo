import { Config } from "@components/config";
import { MaintenanceWindows } from "@components/config/maintenance";
import { ConfigList } from "@components/config/util";
import {
  useInvalidate,
  useLocalStorage,
  usePermissions,
  useRead,
  useWrite,
} from "@lib/hooks";
import { Types } from "komodo_client";
import { ReactNode } from "react";

export const ServerConfig = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) => {
  const { canWrite } = usePermissions({ type: "Server", id });
  const invalidate = useInvalidate();
  const config = useRead("GetServer", { server: id }).data?.config;
  const global_disabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const [update, set] = useLocalStorage<Partial<Types.ServerConfig>>(
    `server-${id}-update-v1`,
    {}
  );
  const { mutateAsync } = useWrite("UpdateServer", {
    onSuccess: () => {
      // In case of disabling to resolve unreachable alert
      invalidate(["ListAlerts"]);
    },
  });
  if (!config) return null;

  const disabled = global_disabled || !canWrite;

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
            label: "Enabled",
            labelHidden: true,
            components: {
              enabled: {
                description:
                  "Whether to attempt to connect to this host / send alerts if offline. Disabling will also convert all attached resource's state to 'Unknown'.",
              },
            },
          },
          {
            label: "Address",
            labelHidden: true,
            components: {
              address: {
                description:
                  "The http/s address of periphery in your network, eg. https://12.34.56.78:8120",
                placeholder: "https://12.34.56.78:8120",
              },
              external_address: {
                description:
                  "Optional. The address of the server used in container links, if different than the Address.",
                placeholder: "https://my.server.int",
              },
              region: {
                placeholder: "Region. Optional.",
                description:
                  "Attach a region to the server for visual grouping.",
              },
            },
          },
          {
            label: "Timeout",
            labelHidden: true,
            components: {
              timeout_seconds: {
                description:
                  "The timeout used with the server health check, in seconds.",
              },
            },
          },
          {
            label: "Disks",
            labelHidden: true,
            components: {
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
            components: {
              stats_monitoring: {
                label: "System Stats Monitoring",
                // boldLabel: true,
                description:
                  "Whether to store historical CPU, RAM, and disk usage.",
              },
            },
          },
          {
            label: "Pruning",
            labelHidden: true,
            components: {
              auto_prune: {
                label: "Auto Prune Images",
                // boldLabel: true,
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
            components: {
              send_unreachable_alerts: {
                // boldLabel: true,
                description:
                  "Send an alert if the Periphery agent cannot be reached.",
              },
            },
          },
          {
            label: "Version",
            labelHidden: true,
            components: {
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
            components: {
              send_cpu_alerts: {
                label: "Send CPU Alerts",
                // boldLabel: true,
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
            components: {
              send_mem_alerts: {
                label: "Send Memory Alerts",
                // boldLabel: true,
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
            components: {
              send_disk_alerts: {
                // boldLabel: true,
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
            boldLabel: false,
            description: (
              <>
                Configure maintenance windows to temporarily disable alerts
                during scheduled maintenance periods. When a maintenance window
                is active, alerts from this server will be suppressed.
              </>
            ),
            components: {
              maintenance_windows: (values, set) => {
                return (
                  <MaintenanceWindows
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
};
