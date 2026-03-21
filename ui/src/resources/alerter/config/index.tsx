import { usePermissions, useRead, useWrite } from "@/lib/hooks";
import Config from "@/ui/config";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import AlerterConfigEndpoint from "./endpoint";
import AlerterConfigAlertTypes from "./alert-types";
import AlerterConfigResources from "./resources";
import ConfigMaintenanceWindows from "@/components/maintenance-windows";
import { useFullAlerter } from "..";

export default function AlerterConfig({ id }: { id: string }) {
  const { canWrite } = usePermissions({ type: "Alerter", id });
  const config = useFullAlerter(id)?.config;
  const global_disabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const { mutateAsync } = useWrite("UpdateAlerter");
  const [update, setUpdate] = useLocalStorage<Partial<Types.AlerterConfig>>({
    key: `alerter-${id}-update-v1`,
    defaultValue: {},
  });

  if (!config) return null;
  const disabled = global_disabled || !canWrite;

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
            label: "Enabled",
            labelHidden: true,
            fields: {
              enabled: {
                boldLabel: true,
                description: "Whether to send alerts to the endpoint.",
              },
            },
          },
          {
            label: "Endpoint",
            labelHidden: true,
            fields: {
              endpoint: (endpoint, set) => (
                <AlerterConfigEndpoint
                  endpoint={endpoint!}
                  set={(endpoint) => set({ endpoint })}
                  disabled={disabled}
                />
              ),
            },
          },
          {
            label: "Filter",
            labelHidden: true,
            fields: {
              alert_types: (alertTypes, set) => (
                <AlerterConfigAlertTypes
                  alertTypes={alertTypes!}
                  set={(alert_types) => set({ alert_types })}
                  disabled={disabled}
                />
              ),
              resources: (resources, set) => (
                <AlerterConfigResources
                  resources={resources!}
                  set={(resources) => set({ resources })}
                  disabled={disabled}
                  blacklist={false}
                />
              ),
              except_resources: (resources, set) => (
                <AlerterConfigResources
                  resources={resources!}
                  set={(except_resources) => set({ except_resources })}
                  disabled={disabled}
                  blacklist={true}
                />
              ),
            },
          },
          {
            label: "Maintenance",
            boldLabel: false,
            description: (
              <>
                Configure maintenance windows to temporarily disable alerts
                during scheduled maintenance periods. When a maintenance window
                is active, alerts which would be sent by this alerter will be
                suppressed.
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
