import { useResourceSelectionState } from "@/lib/hooks";
import ResourceLink from "@/resources/link";
import { DataTable, fmtRateBytes, SortableHeader } from "mogh_ui";
import { BoxProps, Group, Text } from "@mantine/core";
import { Types } from "komodo_client";
import { useServerStats, useServerThresholds } from "@/resources/server/hooks";
import { StatCell } from "mogh_ui";
import ServerVersion from "@/resources/server/version";
import ServerDiskUsage from "../diskUsage";

const SORT_KEYS = ["Name", "Cpu", "Memory", "Disk", "LoadAverage", "Network"];

export default function StatsServerTable({
  resources,
  onServerSort,
  disableSorting,
  ...boxProps
}: {
  resources: Types.ServerListItem[];
  /** When provided, sorting is handled server side,
   * and sort updates are passed to this callback. */
  onServerSort?: (sort: { sort_by?: string; sort_desc?: boolean }) => void;
  disableSorting?: boolean;
} & BoxProps) {
  const selectionState = useResourceSelectionState("Server");
  return (
    <DataTable
      {...boxProps}
      manualSorting={!!onServerSort}
      onSortingStateChange={
        onServerSort &&
        ((sorting) => {
          const sort = sorting.find((s) => SORT_KEYS.includes(s.id));
          onServerSort(
            sort
              ? {
                  sort_by: sort.id,
                  // Descending first
                  sort_desc: !sort.desc,
                }
              : {},
          );
        })
      }
      tableKey="monitoring-server-table"
      data={resources}
      selectOptions={{
        selectKey: ({ name }) => name,
        state: selectionState,
      }}
      columns={[
        {
          id: "Name",
          accessorKey: "name",
          header: ({ column }) => (
            <SortableHeader
              column={column}
              title="Name"
              disabled={disableSorting}
            />
          ),
          cell: ({ row }) => (
            <ResourceLink type="Server" id={row.original.id} />
          ),
        },
        {
          id: "Cpu",
          accessorKey: "id",
          // The stats are fetched per row on the client,
          // sorting is only available server side.
          enableSorting: !!onServerSort,
          header: ({ column }) => (
            <SortableHeader
              column={column}
              title="CPU"
              disabled={disableSorting}
            />
          ),
          cell: ({ row }) => <CpuCell server={row.original} />,
        },
        {
          id: "Memory",
          accessorKey: "id",
          enableSorting: !!onServerSort,
          header: ({ column }) => (
            <SortableHeader
              column={column}
              title="Memory"
              disabled={disableSorting}
            />
          ),
          cell: ({ row }) => <MemCell server={row.original} />,
        },
        {
          id: "Disk",
          accessorKey: "id",
          enableSorting: !!onServerSort,
          header: ({ column }) => (
            <SortableHeader
              column={column}
              title="Disk"
              disabled={disableSorting}
            />
          ),
          cell: ({ row }) => <DiskCell server={row.original} />,
        },
        {
          id: "LoadAverage",
          accessorKey: "id",
          enableSorting: !!onServerSort,
          header: ({ column }) => (
            <SortableHeader
              column={column}
              title="Load Avg"
              disabled={disableSorting}
            />
          ),
          cell: ({ row }) => <LoadAvgCell server={row.original} />,
        },
        {
          id: "Network",
          accessorKey: "id",
          enableSorting: !!onServerSort,
          header: ({ column }) => (
            <SortableHeader
              column={column}
              title="Net"
              disabled={disableSorting}
            />
          ),
          cell: ({ row }) => <NetCell server={row.original} />,
        },
        {
          header: "Version",
          cell: ({ row }) => <ServerVersion id={row.original.id} />,
        },
      ]}
    />
  );
}

function CpuCell({ server }: { server: Types.ServerListItem }) {
  const stats = server.info.stats;
  const cpu = stats?.cpu_perc ?? 0;
  const { cpuWarning: warning, cpuCritical: critical } = useServerThresholds(
    server.id,
  );
  const intent: "Good" | "Warning" | "Critical" =
    cpu < warning ? "Good" : cpu < critical ? "Warning" : "Critical";
  return <StatCell value={stats ? cpu : undefined} intent={intent} />;
}

function MemCell({ server }: { server: Types.ServerListItem }) {
  const stats = server.info.stats;
  const used = stats?.mem_used_gb ?? 0;
  const total = stats?.mem_total_gb ?? 0;
  const perc = total > 0 ? (used / total) * 100 : 0;
  const { memWarning: warning, memCritical: critical } = useServerThresholds(
    server.id,
  );
  const intent: "Good" | "Warning" | "Critical" =
    perc < warning ? "Good" : perc < critical ? "Warning" : "Critical";
  return <StatCell value={stats ? perc : undefined} intent={intent} />;
}

function DiskCell({ server }: { server: Types.ServerListItem }) {
  const stats = server.info.stats;
  const used = stats?.disk_used_gb ?? 0;
  const total = stats?.disk_total_gb ?? 0;
  const perc = total > 0 ? (used / total) * 100 : 0;
  const { diskWarning: warning, diskCritical: critical } = useServerThresholds(
    server.id,
  );
  const intent: "Good" | "Warning" | "Critical" =
    perc < warning ? "Good" : perc < critical ? "Warning" : "Critical";
  return (
    <StatCell
      value={stats ? perc : undefined}
      intent={intent}
      infoDisabled={!stats}
      info={<DiskUsageInfo id={server.id} />}
    />
  );
}

/**
 * The individual disk list is not included in the list item stats.
 * Only mounted when the disk hover card opens, so the full
 * GetSystemStats query is only made on demand.
 */
function DiskUsageInfo({ id }: { id: string }) {
  const stats = useServerStats(id);
  return <ServerDiskUsage id={id} stats={stats} />;
}

function LoadAvgCell({ server }: { server: Types.ServerListItem }) {
  const stats = server.info.stats;
  const one = stats?.load_average?.one;
  const five = stats?.load_average?.five;
  const fifteen = stats?.load_average?.fifteen;
  return (
    <Group gap="xs" wrap="nowrap">
      <Group gap="0.2rem" wrap="nowrap">
        <Text c="dimmed" size="sm">
          1m
        </Text>
        <Text c={one !== undefined ? undefined : "dimmed"}>
          {one !== undefined ? one.toFixed(2) : "N/A"}
        </Text>
      </Group>
      <Group gap="0.2rem" wrap="nowrap">
        <Text c="dimmed" size="sm">
          5m
        </Text>
        <Text c={five !== undefined ? undefined : "dimmed"}>
          {five !== undefined ? five.toFixed(2) : "N/A"}
        </Text>
      </Group>
      <Group gap="0.2rem" wrap="nowrap">
        <Text c="dimmed" size="sm">
          15m
        </Text>
        <Text c={fifteen !== undefined ? undefined : "dimmed"}>
          {fifteen !== undefined ? fifteen.toFixed(2) : "N/A"}
        </Text>
      </Group>
    </Group>
  );
}

function NetCell({ server }: { server: Types.ServerListItem }) {
  const stats = server.info.stats;
  const ingress = stats?.network_ingress_bytes ?? 0;
  const egress = stats?.network_egress_bytes ?? 0;
  if (!stats) {
    return <Text c="dimmed">N/A</Text>;
  }
  return (
    <Text style={{ textWrap: "nowrap" }}>{fmtRateBytes(ingress + egress)}</Text>
  );
}
