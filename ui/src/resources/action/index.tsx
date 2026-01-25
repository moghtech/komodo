import { actionStateIntention, hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { RequiredResourceComponents } from "..";

const useAction = (id?: string) =>
  useRead("ListActions", {}).data?.find((r) => r.id === id);

const ActionIcon = ({ id, size }: { id?: string; size: string | number }) => {
  const state = useAction(id)?.info.state;
  const color = state && hexColorByIntention(actionStateIntention(state));
  return <ICONS.Action size={size} color={color} />;
};

export const ActionComponents: RequiredResourceComponents = {
  useListItem: useAction,
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
  Table: () => <></>,

  Icon: ({ id }) => <ActionIcon id={id} size="1rem" />,
  BigIcon: ({ id }) => <ActionIcon id={id} size="1.4rem" />,
  State: ({ id }) => <></>,
  Config: ({ id }) => <></>,
  DangerZone: ({ id }) => <></>,

  Status: {},
  Info: {},
  Actions: {},
  Page: {},
};
