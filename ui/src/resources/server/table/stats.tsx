import { useSelectedResources } from "@/lib/hooks";
import ResourceLink from "@/resources/link";
import { DataTable, fmtRateBytes, SortableHeader } from "mogh_ui";
import { BoxProps, Text, Group } from "@mantine/core";
import { Types } from "komodo_client";
import { useServerStats, useServerThresholds } from "@/resources/server/hooks";
import StatCell from "@/ui/stat-cell";
import ServerVersion from "@/resources/server/version";
import ServerDiskUsage from "../diskUsage";
import { ICONS } from "@/lib/icons";

export default function StatsServerTable({
  resources,
  noBorder,
  ...boxProps
}: {
  resources: Types.ServerListItem[];
  noBorder?: boolean;
} & BoxProps) {
  const [_, setSelectedResources] = useSelectedResources("Server");
  return (
    <DataTable
      {...boxProps}
      noBorder={noBorder}
      className={`monitoring-stats-table ${!noBorder ? "bordered-light" : ""} ${boxProps.className ?? ""}`}
      tableKey="monitoring-server-table"
      data={resources}
      selectOptions={{
        selectKey: ({ name }) => name,
        onSelect: setSelectedResources,
      }}
      columns={[
        {
          size: 240,
          accessorKey: "name",
          header: ({ column }) => (
            <SortableHeader column={column} title="Name" />
          ),
          cell: ({ row }) => <ResourceLink type="Server" id={row.original.id} />,
        },
        {
          header: "CPU",
          size: 170,
          cell: ({ row }) => <CpuCell id={row.original.id} />,
        },
        {
          header: "Temp",
          size: 170,
          cell: ({ row }) => <TempCell id={row.original.id} />,
        },
        {
          header: "Memory",
          size: 170,
          cell: ({ row }) => <MemCell id={row.original.id} />,
        },
        {
          header: "Disk",
          size: 170,
          cell: ({ row }) => <DiskCell id={row.original.id} />,
        },
        {
          header: "Load Avg",
          size: 210,
          cell: ({ row }) => <LoadAvgCell id={row.original.id} />,
        },
        {
          header: "Net",
          size: 120,
          cell: ({ row }) => <NetCell id={row.original.id} />,
        },
        {
          header: "Version",
          size: 121,
          cell: ({ row }) => <ServerVersion id={row.original.id} />,
        },
      ]}
    />
  );
}

function CpuCell({ id }: { id: string }) {
  const stats = useServerStats(id);
  const thresholds = useServerThresholds(id);
  const value = stats?.cpu_perc ?? 0;

  const intent: "Good" | "Warning" | "Critical" =
    value < thresholds.cpuWarning
      ? "Good"
      : value < thresholds.cpuCritical
        ? "Warning"
        : "Critical";

  return <StatCell value={stats ? value : undefined} intent={intent} />;
}

function TempCell({ id }: { id: string }) {
  const stats = useServerStats(id);
  const value = stats?.cpu_temp;

  const intent: "Good" | "Warning" | "Critical" =
    value === undefined
      ? "Good"
      : value < 65
        ? "Good"
        : value < 80
          ? "Warning"
          : "Critical";

  return (
    <StatCell value={stats ? value : undefined} intent={intent} suffix="°C" />
  );
}

function MemCell({ id }: { id: string }) {
  const stats = useServerStats(id);
  const thresholds = useServerThresholds(id);

  const used = stats?.mem_used_gb ?? 0;
  const total = stats?.mem_total_gb ?? 0;
  const value = total > 0 ? (used / total) * 100 : 0;

  const intent: "Good" | "Warning" | "Critical" =
    value < thresholds.memWarning
      ? "Good"
      : value < thresholds.memCritical
        ? "Warning"
        : "Critical";

  return <StatCell value={stats ? value : undefined} intent={intent} />;
}

function DiskCell({ id }: { id: string }) {
  const stats = useServerStats(id);
  const thresholds = useServerThresholds(id);

  const used = stats?.disks?.reduce((acc, d) => acc + (d.used_gb || 0), 0) ?? 0;
  const total =
    stats?.disks?.reduce((acc, d) => acc + (d.total_gb || 0), 0) ?? 0;
  const value = total > 0 ? (used / total) * 100 : 0;

  const intent: "Good" | "Warning" | "Critical" =
    value < thresholds.diskWarning
      ? "Good"
      : value < thresholds.diskCritical
        ? "Warning"
        : "Critical";

  return (
    <StatCell
      value={stats ? value : undefined}
      intent={intent}
      infoDisabled={!stats}
      info={<ServerDiskUsage id={id} stats={stats} />}
    />
  );
}

function LoadAvgCell({ id }: { id: string }) {
  const stats = useServerStats(id);
  const one = stats?.load_average?.one;
  const five = stats?.load_average?.five;
  const fifteen = stats?.load_average?.fifteen;
  return (
    <Group gap="sm" wrap="nowrap">
      <Group gap={4} wrap="nowrap">
        <Text component="span" c="dimmed" style={{ fontSize: "11px" }}>
          1m
        </Text>
        <Text size="sm">{one !== undefined ? one.toFixed(2) : "N/A"}</Text>
      </Group>
      <Group gap={4} wrap="nowrap">
        <Text component="span" c="dimmed" style={{ fontSize: "11px" }}>
          5m
        </Text>
        <Text size="sm">{five !== undefined ? five.toFixed(2) : "N/A"}</Text>
      </Group>
      <Group gap={4} wrap="nowrap">
        <Text component="span" c="dimmed" style={{ fontSize: "11px" }}>
          15m
        </Text>
        <Text size="sm">{fifteen !== undefined ? fifteen.toFixed(2) : "N/A"}</Text>
      </Group>
    </Group>
  );
}

function NetCell({ id }: { id: string }) {
  const stats = useServerStats(id);
  const ingress = stats?.network_ingress_bytes ?? 0;
  const egress = stats?.network_egress_bytes ?? 0;
  if (!stats) {
    return (
      <Text c="dimmed" size="sm">
        N/A
      </Text>
    );
  }
  return (
    <Group gap="xs" wrap="nowrap">
      <ICONS.Network size="1.1rem" />
      <Text size="sm">{fmtRateBytes(ingress + egress)}</Text>
    </Group>
  );
}
