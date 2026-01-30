import { stackStateIntention, hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import ResourceHeader from "@/components/resource-header";
import StackTable from "./table";
import NewResource from "@/resources/new";

export const StackComponents: RequiredResourceComponents<
  Types.StackConfig,
  Types.StackInfo,
  Types.StackListItemInfo
> = {
  useList: () => useRead("ListStacks", {}).data,
  useListItem: (id) => StackComponents.useList()?.find((r) => r.id === id),

  useFull: (id) => useRead("GetStack", { stack: id }).data,

  useResourceLinks: (stack) => stack?.config?.links,

  useDashboardSummaryData: () => {
    const summary = useRead("GetStacksSummary", {}).data;
    const all = [
      summary?.running ?? 0,
      summary?.stopped ?? 0,
      summary?.unhealthy ?? 0,
      summary?.unknown ?? 0,
    ];
    const [running, stopped, unhealthy, unknown] = all;
    return [
      all.every((item) => item === 0) && {
        title: "Down",
        intention: "Neutral",
        value: summary?.down ?? 0,
      },
      { intention: "Good", value: running, title: "Running" },
      {
        intention: "Warning",
        value: stopped,
        title: "Stopped",
      },
      {
        intention: "Critical",
        value: unhealthy,
        title: "Unhealthy",
      },
      {
        intention: "Unknown",
        value: unknown,
        title: "Unknown",
      },
    ];
  },

  Description: () => <>Deploy docker compose files.</>,

  New: () => <NewResource type="Stack" />,

  GroupExecutions: () => <></>,

  Table: StackTable,

  Icon: ({ id, size = "1rem" }) => {
    const state = useRead("ListStacks", {}).data?.find((r) => r.id === id)?.info
      .state;
    const color = state && hexColorByIntention(stackStateIntention(state));
    return <ICONS.Stack size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const stack = StackComponents.useListItem(id);
    return (
      <ResourceHeader
        intent={stackStateIntention(stack?.info.state)}
        icon={<StackComponents.Icon id={id} size="2rem" />}
        name={stack?.name}
        state={stack?.info.state}
        status={
          stack?.info.state === Types.StackState.Unhealthy
            ? stack?.info.status
            : undefined
        }
      />
    );
  },

  State: ({ id }) => {
    let state = StackComponents.useListItem(id)?.info.state;
    return <StatusBadge text={state} intent={stackStateIntention(state)} />;
  },
  Status: {},
  Info: {},

  Executions: {},

  Config: () => <>CONFIG</>,
  DangerZone: ({ id }) => <></>,

  Page: {},
};
