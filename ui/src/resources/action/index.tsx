import { actionStateIntention, hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { RequiredResourceComponents } from "..";
import { ActionTable } from "./table";
import { Types } from "komodo_client";
import StatusBadge from "@/components/common/status-badge";

export const ActionComponents: RequiredResourceComponents = {
  useListItem: (id) =>
    useRead("ListActions", {}).data?.find((r) => r.id === id),

  useFull: (id) => useRead("GetAction", { action: id }).data,

  useResourceLinks: () => undefined,

  useDashboardSummaryData: () => {
    const summary = useRead("GetActionsSummary", {}).data;
    return [
      { title: "Ok", intention: "Good", value: summary?.ok ?? 0 },
      {
        title: "Running",
        intention: "Warning",
        value: summary?.running ?? 0,
      },
      {
        title: "Failed",
        intention: "Critical",
        value: summary?.failed ?? 0,
      },
      {
        title: "Unknown",
        intention: "Unknown",
        value: summary?.unknown ?? 0,
      },
    ];
  },

  Description: () => <>Custom scripts using the Komodo client.</>,

  ResourcePageHeader: () => <></>,

  New: () => <></>,

  GroupActions: () => <></>,

  Table: ({ resources }) => (
    <ActionTable actions={resources as Types.ActionListItem[]} />
  ),

  Icon: ({ id, size = "1rem" }) => {
    const state = useRead("ListActions", {}).data?.find((r) => r.id === id)
      ?.info.state;
    const color = state && hexColorByIntention(actionStateIntention(state));
    return <ICONS.Action size={size} color={color} />;
  },

  State: ({ id }) => {
    let state = (
      ActionComponents.useListItem(id)?.info as Types.ActionListItemInfo
    ).state;
    return <StatusBadge text={state} intent={actionStateIntention(state)} />;
  },

  Config: ({ id }) => <></>,
  DangerZone: ({ id }) => <></>,

  Status: {},
  Info: {},
  Actions: {},
  Page: {},
};
