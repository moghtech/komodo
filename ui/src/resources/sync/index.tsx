import { resourceSyncStateIntention, hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import ResourceSyncTable from "./table";
import { RequiredResourceComponents } from "@/resources";
import NewResource from "@/resources/new";
import { CommitSync, ExecuteSync, RefreshSync } from "./executions";
import FileSource from "@/components/file-source";
import { Clock } from "lucide-react";
import { fmtDate } from "@/lib/formatting";
import { Box, Group } from "@mantine/core";
import HashCompare from "@/components/hash-compare";
import ResourceSyncTabs from "./tabs";
import ResourceHeader from "../header";
import BatchExecutions from "@/components/batch-executions";

export function useResourceSync(id: string | undefined, useName?: boolean) {
  return useRead("ListResourceSyncs", {}).data?.find((r) =>
    useName ? r.name === id : r.id === id,
  );
}

export function useFullResourceSync(id: string) {
  return useRead("GetResourceSync", { sync: id }, { refetchInterval: 30_000 })
    .data;
}

export const ResourceSyncComponents: RequiredResourceComponents<
  Types.ResourceSyncConfig,
  Types.ResourceSyncInfo,
  Types.ResourceSyncListItemInfo
> = {
  useList: () => useRead("ListResourceSyncs", {}).data,
  useListItem: useResourceSync,
  useFull: useFullResourceSync,

  useResourceLinks: () => undefined,

  useDashboardSummaryData: () => {
    const summary = useRead(
      "GetResourceSyncsSummary",
      {},
      { refetchInterval: 10_000 },
    ).data;
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

  New: () => <NewResource type="ResourceSync" readableType="Sync" />,

  BatchExecutions: () => (
    <BatchExecutions
      type="ResourceSync"
      executions={[
        ["RunSync", ICONS.Run],
        ["CommitSync", ICONS.Commit],
      ]}
    />
  ),

  Table: ResourceSyncTable,

  Icon: ({ id, size = "1rem", noColor }) => {
    const state = useRead("ListResourceSyncs", {}).data?.find(
      (r) => r.id === id,
    )?.info.state;
    const color = noColor
      ? undefined
      : state && hexColorByIntention(resourceSyncStateIntention(state));
    return <ICONS.ResourceSync size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const resourceSync = useResourceSync(id);
    return (
      <ResourceHeader
        type="ResourceSync"
        id={id}
        resource={resourceSync}
        intent={resourceSyncStateIntention(resourceSync?.info.state)}
        icon={ICONS.ResourceSync}
        name={resourceSync?.name}
        state={resourceSync?.info.state}
      />
    );
  },

  State: ({ id }) => {
    let state = useResourceSync(id)?.info.state;
    return (
      <StatusBadge text={state} intent={resourceSyncStateIntention(state)} />
    );
  },

  Info: {
    Source: ({ id }) => {
      const info = useResourceSync(id)?.info;
      return <FileSource info={info} />;
    },
    LastSync: ({ id }) => {
      const last_ts = useResourceSync(id)?.info.last_sync_ts;
      return (
        <Group gap="xs">
          <Clock size="1rem" />
          {last_ts ? fmtDate(new Date(last_ts)) : "Never"}
        </Group>
      );
    },
    Hash: ({ id }) => {
      const info = useFullResourceSync(id)?.info;
      if (!info?.pending_hash) {
        return null;
      }
      return (
        <Box>
          <HashCompare
            lastHash={info?.last_sync_hash}
            lastMessage={info?.last_sync_message}
            lastLabel="synced"
            latestHash={info?.pending_hash}
            latestMessage={info?.pending_message}
          />
        </Box>
      );
    },
  },

  Executions: {
    RefreshSync,
    ExecuteSync: ExecuteSync,
    CommitSync,
  },

  Config: ResourceSyncTabs,

  Page: {},
};
