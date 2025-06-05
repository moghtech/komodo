import { AlertsTable } from "@components/alert/table";
import { Page } from "@components/layouts";
import { useRead } from "@lib/hooks";
import { Types } from "komodo_client";
import { Button } from "@ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { Switch } from "@ui/switch";
import {
  AlertTriangle,
  Box,
  ChevronLeft,
  ChevronRight,
  MinusCircle,
} from "lucide-react";
import { useMemo, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { UsableResource } from "@types";
import { SelectSeparator } from "@radix-ui/react-select";
import { ResourceComponents } from "@components/resources";
import { ResourceSelector } from "@components/resources/common";

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

export default function AlertsPage() {
  const [page, setPage] = useState(0);
  const [params, setParams] = useSearchParams();

  const { type, id, alert_type, open } = useMemo(
    () => ({
      type: (params.get("type") as UsableResource) ?? undefined,
      id: params.get("id") ?? undefined,
      alert_type: (params.get("alert") as Types.AlertData["type"]) ?? undefined,
      open: params.get("open") === "true" || undefined,
    }),
    [params]
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
      icon={<AlertTriangle className="w-8" />}
      actions={
        <>
          <div className="flex items-center md:justify-end gap-4 flex-wrap">
            {/* resource type */}
            <Select
              value={type ?? "All"}
              onValueChange={(type) => {
                const p = new URLSearchParams(params.toString());
                type === "all" ? p.delete("type") : p.set("type", type);
                p.delete("id");
                p.delete("operation");
                setParams(p);
              }}
            >
              <SelectTrigger className="w-48">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="All">
                  <div className="flex items-center gap-2">
                    <Box className="w-4 text-muted-foreground" />
                    All Resources
                  </div>
                </SelectItem>
                <SelectSeparator />
                {Object.keys(ALERT_TYPES_BY_RESOURCE).map((type) => {
                  const Icon = ResourceComponents[type].Icon;
                  return (
                    <SelectItem key={type} value={type}>
                      <div className="flex items-center gap-2">
                        <span className="text-muted-foreground">
                          <Icon />
                        </span>
                        {type}
                      </div>
                    </SelectItem>
                  );
                })}
              </SelectContent>
            </Select>

            {/* resource id */}
            {type && (
              <ResourceSelector
                type={type}
                selected={id}
                onSelect={(id) => {
                  const p = new URLSearchParams(params.toString());
                  id === "all" ? p.delete("id") : p.set("id", id);
                  setParams(p);
                }}
              />
            )}

            {/* operation */}
            <Select
              value={alert_type ?? "All"}
              onValueChange={(alert) => {
                const p = new URLSearchParams(params.toString());
                alert === "All" ? p.delete("alert") : p.set("alert", alert);
                setParams(p);
              }}
            >
              <SelectTrigger className="w-64 overflow-ellipsis">
                <SelectValue placeholder="Alert Type" />
              </SelectTrigger>
              <SelectContent align="end">
                <SelectItem value="All">
                  <div className="flex items-center gap-2">All Alerts</div>
                </SelectItem>
                {alert_types.map((variant) => (
                  <SelectItem key={variant} value={variant}>
                    {variant}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            {/* only open */}
            <div
              className="px-4 h-9 flex items-center gap-4 border rounded-md"
              onClick={() => {
                const p = new URLSearchParams(params.toString());
                open ? p.delete("open") : p.set("open", "true");
                setParams(p);
              }}
            >
              <p className="text-sm text-muted-foreground">Only Open</p>
              <Switch checked={open} className="pointer-events-none" />
            </div>

            {/* reset */}
            <Button
              size="icon"
              onClick={() => setParams({})}
              variant="secondary"
            >
              <MinusCircle className="w-4" />
            </Button>
          </div>
        </>
      }
    >
      <div className="flex flex-col gap-2">
        <AlertsTable alerts={alerts?.alerts ?? []} showResolved />
        <div className="flex gap-4 items-center">
          <Button
            variant="outline"
            onClick={() => setPage(page - 1)}
            disabled={page === 0}
            size="icon"
          >
            <ChevronLeft className="w-4" />
          </Button>
          {Array.from(new Array(page + 1)).map((_, i) => (
            <Button
              key={i}
              onClick={() => setPage(i)}
              variant={page === i ? "secondary" : "outline"}
            >
              {i + 1}
            </Button>
          ))}
          {/* Page: {page + 1} */}
          <Button
            variant="outline"
            onClick={() => alerts?.next_page && setPage(alerts.next_page)}
            disabled={!alerts?.next_page}
            size="icon"
          >
            <ChevronRight className="w-4" />
          </Button>
        </div>
      </div>
    </Page>
  );
}
