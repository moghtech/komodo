import { swarmStateIntention, hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import EntityHeader from "@/ui/entity-header";
import SwarmTable from "./table";
import NewResource from "@/resources/new";
import DeleteResource from "../delete";

export function useSwarm(id: string | undefined) {
  return useRead("ListSwarms", {}).data?.find((r) => r.id === id);
}

export function useFullSwarm(id: string) {
  return useRead("GetSwarm", { swarm: id }).data;
}

export const SwarmComponents: RequiredResourceComponents<
  Types.SwarmConfig,
  Types.SwarmInfo,
  Types.SwarmListItemInfo
> = {
  useList: () => useRead("ListSwarms", {}).data,
  useListItem: useSwarm,
  useFull: useFullSwarm,

  useResourceLinks: (swarm) => swarm?.config?.links,

  useDashboardSummaryData: () => {
    const summary = useRead("GetSwarmsSummary", {}).data;
    return [
      { intention: "Good", value: summary?.healthy ?? 0, title: "Healthy" },
      {
        intention: "Critical",
        value: (summary?.unhealthy ?? 0) + (summary?.down ?? 0),
        title: "Unhealthy",
      },
      {
        intention: "Unknown",
        value: summary?.unknown ?? 0,
        title: "Unknown",
      },
    ];
  },

  Description: () => <>Control and monitor docker swarms.</>,

  New: () => <NewResource type="Swarm" />,

  GroupExecutions: () => <></>,

  Table: SwarmTable,

  Icon: ({ id, size = "1rem" }) => {
    const state = useRead("ListSwarms", {}).data?.find((r) => r.id === id)?.info
      .state;
    const color = state && hexColorByIntention(swarmStateIntention(state));
    return <ICONS.Swarm size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const swarm = useSwarm(id);
    return (
      <EntityHeader
        intent={swarmStateIntention(swarm?.info.state)}
        icon={ICONS.Swarm}
        name={swarm?.name}
        state={swarm?.info.state}
        action={<DeleteResource type="Swarm" id={id} />}
      />
    );
  },

  State: ({ id }) => {
    let state = useSwarm(id)?.info.state;
    return <StatusBadge text={state} intent={swarmStateIntention(state)} />;
  },
  Info: {},

  Executions: {},

  Config: () => <>CONFIG</>,

  Page: {},
};
