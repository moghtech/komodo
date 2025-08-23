import { ConfigItem } from "@components/config/util";
import { Types } from "komodo_client";
import { Badge } from "@ui/badge";
import { Select, SelectContent, SelectItem, SelectTrigger } from "@ui/select";
import { MinusCircle } from "lucide-react";

const ALERT_TYPES: Types.AlertData["type"][] = [
  // Server
  "ServerVersionMismatch",
  "ServerUnreachable",
  "ServerCpu",
  "ServerMem",
  "ServerDisk",
  // Stack
  "StackStateChange",
  "StackImageUpdateAvailable",
  "StackAutoUpdated",
  // Deployment
  "ContainerStateChange",
  "DeploymentImageUpdateAvailable",
  "DeploymentAutoUpdated",
  // Misc
  "ScheduleRun",
  "BuildFailed",
  "ResourceSyncPendingUpdates",
  "RepoBuildFailed",
  "ActionFailed",
  "ProcedureFailed",
  "AwsBuilderTerminationFailed",
  "Custom",
];

export const AlertTypeConfig = ({
  alert_types,
  set,
  disabled,
}: {
  alert_types: Types.AlertData["type"][];
  set: (alert_types: Types.AlertData["type"][]) => void;
  disabled: boolean;
}) => {
  const at = ALERT_TYPES.filter(
    (alert_type) => !alert_types.includes(alert_type),
  );
  return (
    <ConfigItem
      label="Alert Types"
      description="Only send alerts of certain types."
      boldLabel
    >
      <div className="flex items-center gap-4">
        {at.length ? (
          <Select
            value={undefined}
            onValueChange={(type: Types.AlertData["type"]) => {
              set([...alert_types, type]);
            }}
            disabled={disabled}
          >
            <SelectTrigger className="w-[150px]">
              <div className="pr-2">Add Filter</div>
            </SelectTrigger>
            <SelectContent align="start">
              {at.map((alert_type) => (
                <SelectItem key={alert_type} value={alert_type}>
                  {alert_type}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        ) : undefined}
        <div className="flex items-center flex-wrap gap-2 w-[75%]">
          {alert_types.map((type) => (
            <Badge
              variant="secondary"
              className="text-sm flex items-center gap-2 cursor-pointer"
              onClick={() => {
                if (disabled) return;
                set(alert_types.filter((t) => t !== type));
              }}
            >
              {type}
              {!disabled && <MinusCircle className="w-3 h-3" />}
            </Badge>
          ))}
        </div>
      </div>
    </ConfigItem>
  );
};
