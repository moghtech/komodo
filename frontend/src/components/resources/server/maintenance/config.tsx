import { Section } from "@components/layouts";
import { useInvalidate, useLocalStorage, usePermissions, useRead, useWrite } from "@lib/hooks";
import { Types } from "komodo_client";
import { ReactNode } from "react";
import { MaintenanceWindows } from "./modal";
import { Button } from "@ui/button";
import { Save } from "lucide-react";

export const MaintenanceServerConfig = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) => {
  const invalidate = useInvalidate();
  const server = useRead("GetServer", { server: id }).data;
  const config = server?.config;
  const global_disabled = useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const { canWrite } = usePermissions({ type: "Server", id });
  
  const [maintenanceWindows, setMaintenanceWindows] = useLocalStorage<Types.MaintenanceWindow[]>(
    `server-${id}-maintenance-windows`,
    config?.maintenance_windows ?? []
  );
  
  const { mutateAsync: updateServer, isPending } = useWrite("UpdateServer", {
    onSuccess: () => {
      invalidate(["GetServer", { server: id }]);
      invalidate(["ListAlerts"]);
    },
  });

  const disabled = global_disabled || !canWrite;
  
  const hasChanges = JSON.stringify(maintenanceWindows) !== JSON.stringify(config?.maintenance_windows ?? []);

  const handleSave = async () => {
    if (!config) return;
    
    await updateServer({
      id,
      config: {
        ...config,
        maintenance_windows: maintenanceWindows,
      },
    });
  };

  const handleReset = () => {
    setMaintenanceWindows(config?.maintenance_windows ?? []);
  };

  if (!config) return null;

  return (
    <Section
      titleOther={titleOther}
      actions={
        hasChanges && !disabled && (
          <div className="flex gap-2">
            <Button variant="outline" onClick={handleReset} disabled={isPending}>
              Reset
            </Button>
            <Button onClick={handleSave} disabled={isPending}>
              <Save className="w-4 h-4 mr-2" />
              Save Changes
            </Button>
          </div>
        )
      }
    >
      <div className="space-y-4">
        <div className="text-sm text-muted-foreground space-y-2">
          <p>Configure maintenance windows to temporarily disable alerts during scheduled maintenance periods. 
          When a maintenance window is active, alerts from this server will be suppressed.</p>
        </div>
        <MaintenanceWindows
          windows={maintenanceWindows}
          onUpdate={setMaintenanceWindows}
          disabled={disabled}
        />
      </div>
    </Section>
  );
};