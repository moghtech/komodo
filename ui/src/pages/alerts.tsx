import { useRead } from "@/lib/hooks";
import { UsableResource } from "@/resources";
import { ICONS } from "@/theme/icons";
import Page from "@/ui/page";
import { Types } from "komodo_client";
import { useMemo, useState } from "react";
import { useSearchParams } from "react-router-dom";

const ALERT_TYPES_BY_RESOURCE: { [key: string]: Types.AlertData["type"][] } = {
  Server: ["ServerUnreachable", "ServerCpu", "ServerMem", "ServerDisk"],
  Stack: ["StackStateChange", "StackImageUpdateAvailable", "StackAutoUpdated"],
  Deployment: [
    "ContainerStateChange",
    "DeploymentImageUpdateAvailable",
    "DeploymentAutoUpdated",
  ],
  Build: ["BuildFailed"],
  Repo: ["RepoBuildFailed"],
  ResourceSync: ["ResourceSyncPendingUpdates"],
};

const FALLBACK_ALERT_TYPES = [
  ...Object.values(ALERT_TYPES_BY_RESOURCE).flat(),
  "AwsBuilderTerminationFailed",
];

export default function Alerts() {
  const [page, setPage] = useState(0);
  const [params, setParams] = useSearchParams();

  const { type, id, alert_type, open } = useMemo(
    () => ({
      type: (params.get("type") as UsableResource) ?? undefined,
      id: params.get("id") ?? undefined,
      alert_type: (params.get("alert") as Types.AlertData["type"]) ?? undefined,
      open: params.get("open") === "true" || undefined,
    }),
    [params],
  );

  const { data: alerts } = useRead("ListAlerts", {
    query: {
      "target.type": type,
      "target.id": id,
      "data.type": alert_type,
      resolved: !open,
    },
    page,
  });

  const alert_types: string[] = type
    ? (ALERT_TYPES_BY_RESOURCE[type] ?? FALLBACK_ALERT_TYPES)
    : FALLBACK_ALERT_TYPES;

  return (
    <Page
      title="Alerts"
      icon={ICONS.Alert}
      description="View historical alerts"
    ></Page>
  );
}
