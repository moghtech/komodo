import { Config } from "@components/config";
import { useLocalStorage, usePermissions, useRead, useWrite } from "@lib/hooks";
import { Types } from "komodo_client";
import { EndpointConfig } from "./endpoint";
import { AlertTypeConfig } from "./alert_types";
import { ResourcesConfig } from "./resources";
import { MaintenanceWindows } from "@components/config/maintenance";

export const AlerterConfig = ({ id }: { id: string }) => {
  const { canWrite } = usePermissions({ type: "Alerter", id });
  const config = useRead("GetAlerter", { alerter: id }).data?.config;
  const global_disabled =
    useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const { mutateAsync } = useWrite("UpdateAlerter");
  const [update, set] = useLocalStorage<Partial<Types.AlerterConfig>>(
    `alerter-${id}-update-v1`,
    {}
  );

  if (!config) return null;
  const disabled = global_disabled || !canWrite;

  return (
    <Config
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
                boldLabel: true,
                description: "Whether to send alerts to the endpoint.",
              },
            },
          },
          {
            label: "Endpoint",
            labelHidden: true,
            components: {
              endpoint: (endpoint, set) => (
                <EndpointConfig
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
            components: {
              alert_types: (alert_types, set) => (
                <AlertTypeConfig
                  alert_types={alert_types!}
                  set={(alert_types) => set({ alert_types })}
                  disabled={disabled}
                />
              ),
              resources: (resources, set) => (
                <ResourcesConfig
                  resources={resources!}
                  set={(resources) => set({ resources })}
                  disabled={disabled}
                  blacklist={false}
                />
              ),
              except_resources: (resources, set) => (
                <ResourcesConfig
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
