import { Section } from "@components/layouts";
import { useInvalidate, useRead, useWrite } from "@lib/hooks";
import { Types } from "komodo_client";
import { ReactNode, useState } from "react";
import { MaintenanceWindows } from "./maintenance-windows";
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
  const perms = useRead("GetPermissionLevel", {
    target: { type: "Server", id },
  }).data;
  
  const [maintenanceWindows, setMaintenanceWindows] = useState<Types.MaintenanceWindow[]>(
    config?.maintenance_windows ?? []
  );
  
  const { mutateAsync: updateServer, isPending } = useWrite("UpdateServer", {
    onSuccess: () => {
      invalidate(["GetServer", { server: id }]);
      invalidate(["ListAlerts"]);
    },
  });

  const disabled = global_disabled || perms !== Types.PermissionLevel.Write;
  
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
      title="Maintenance Windows"
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
      <div className="mb-4 text-muted-foreground">
        Configure scheduled maintenance windows to suppress alerts during planned downtime
      </div>
      <MaintenanceWindows
        windows={maintenanceWindows}
        onUpdate={setMaintenanceWindows}
        disabled={disabled}
        description="Schedule maintenance windows to automatically suppress server alerts during planned maintenance periods."
      />
    </Section>
  );
};