import { resourceSyncStateIntention, hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import ResourceHeader from "@/components/resource-header";

export const ResourceSyncComponents: RequiredResourceComponents<
  Types.ResourceSyncConfig,
  Types.ResourceSyncInfo,
  Types.ResourceSyncListItemInfo
> = {
  useListItem: (id) =>
    useRead("ListResourceSyncs", {}).data?.find((r) => r.id === id),

  useFull: (id) => useRead("GetResourceSync", { sync: id }).data,

  useResourceLinks: () => undefined,

  useDashboardSummaryData: () => {
    const summary = useRead("GetResourceSyncsSummary", {}).data;
    return [
      { title: "Ok", intention: "Good", value: summary?.ok ?? 0 },
      {
        title: "Syncing",
        intention: "Warning",
        value: summary?.syncing ?? 0,
      },
      {
        title: "Pending",
        intention: "Neutral",
        value: summary?.pending ?? 0,
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

  Description: () => <>Declare resources in TOML files.</>,

  New: () => <></>,

  GroupExecutions: () => <></>,

  Table: ({ resources }) => (
    // <ResourceSyncTable resourceSyncs={resources as Types.ResourceSyncListItem[]} />
    <></>
  ),

  Icon: ({ id, size = "1rem" }) => {
    const state = useRead("ListResourceSyncs", {}).data?.find(
      (r) => r.id === id,
    )?.info.state;
    const color =
      state && hexColorByIntention(resourceSyncStateIntention(state));
    return <ICONS.ResourceSync size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const resourceSync = ResourceSyncComponents.useListItem(id);
    return (
      <ResourceHeader
        intent={resourceSyncStateIntention(resourceSync?.info.state)}
        icon={<ResourceSyncComponents.Icon id={id} size="2rem" />}
        name={resourceSync?.name}
        state={resourceSync?.info.state}
      />
    );
  },

  State: ({ id }) => {
    let state = ResourceSyncComponents.useListItem(id)?.info.state;
    return (
      <StatusBadge text={state} intent={resourceSyncStateIntention(state)} />
    );
  },
  Status: {},
  Info: {},

  Executions: {},

  Config: () => <>CONFIG</>,
  DangerZone: ({ id }) => <></>,

  Page: {},
};
