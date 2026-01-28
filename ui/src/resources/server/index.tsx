import { serverStateIntention, hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import ResourceHeader from "@/components/resource-header";
import ServerTable from "./table";

export const ServerComponents: RequiredResourceComponents<
  Types.ServerConfig,
  Types.ServerInfo,
  Types.ServerListItemInfo
> = {
  useListItem: (id) =>
    useRead("ListServers", {}).data?.find((r) => r.id === id),

  useFull: (id) => useRead("GetServer", { server: id }).data,

  useResourceLinks: (server) => server?.config?.links,

  useDashboardSummaryData: () => {
    const summary = useRead("GetServersSummary", {}).data;
    return [
      { title: "Healthy", intention: "Good", value: summary?.healthy ?? 0 },
      {
        title: "Warning",
        intention: "Warning",
        value: summary?.warning ?? 0,
      },
      {
        title: "Unhealthy",
        intention: "Critical",
        value: summary?.unhealthy ?? 0,
      },
      {
        title: "Disabled",
        intention: "Neutral",
        value: summary?.disabled ?? 0,
      },
    ];
  },

  Description: () => (
    <>Connect servers for alerting, building, and deploying.</>
  ),

  New: () => <></>,

  GroupExecutions: () => <></>,

  Table: ServerTable,

  Icon: ({ id, size = "1rem" }) => {
    const state = useRead("ListServers", {}).data?.find((r) => r.id === id)
      ?.info.state;
    const color = state && hexColorByIntention(serverStateIntention(state));
    return <ICONS.Server size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const server = ServerComponents.useListItem(id);
    return (
      <ResourceHeader
        intent={serverStateIntention(server?.info.state)}
        icon={<ServerComponents.Icon id={id} size="2rem" />}
        name={server?.name}
        state={server?.info.state}
        status={server?.info.region}
      />
    );
  },

  State: ({ id }) => {
    let state = ServerComponents.useListItem(id)?.info.state;
    return <StatusBadge text={state} intent={serverStateIntention(state)} />;
  },
  Status: {},
  Info: {},

  Executions: {},

  Config: () => <>CONFIG</>,
  DangerZone: ({ id }) => <></>,

  Page: {},
};
