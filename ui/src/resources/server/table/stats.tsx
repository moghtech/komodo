import { useSelectedResources } from "@/lib/hooks";
import ResourceLink from "@/resources/link";
import { DataTable, SortableHeader } from "@/ui/data-table";
import { BoxProps, Group, Stack, Text } from "@mantine/core";
import { Types } from "komodo_client";
import { useServerStats, useServerThresholds } from "@/resources/server/hooks";
import StatCell from "@/ui/stat-cell";
import { fmtRateBytes } from "@/lib/formatting";
import ServerVersion from "@/resources/server/version";

export default function StatsServerTable({
  resources,
  ...boxProps
}: {
  resources: Types.ServerListItem[];
} & BoxProps) {
  const [_, setSelectedResources] = useSelectedResources("Server");
  return (
    <DataTable
      {...boxProps}
      tableKey="monitoring-server-table"
      data={resources}
      selectOptions={{
        selectKey: ({ name }) => name,
        onSelect: setSelectedResources,
      }}
      columns={[
        {
          size: 250,
          accessorKey: "name",
          header: ({ column }) => (
            <SortableHeader column={column} title="Name" />
          ),
          cell: ({ row }) => (
            <ResourceLink type="Server" id={row.original.id} />
          ),
        },
        {
          header: "CPU",
          size: 200,
          cell: ({ row }) => <CpuCell id={row.original.id} />,
        },
        {
          header: "Memory",
          size: 200,
          cell: ({ row }) => <MemCell id={row.original.id} />,
        },
        {
          header: "Disk",
          size: 200,
          cell: ({ row }) => <DiskCell id={row.original.id} />,
        },
        {
          header: "Load Avg",
          size: 160,
          cell: ({ row }) => <LoadAvgCell id={row.original.id} />,
        },
        {
          header: "Net",
          size: 100,
          cell: ({ row }) => <NetCell id={row.original.id} />,
        },
        {
          header: "Version",
          size: 160,
          cell: ({ row }) => <ServerVersion id={row.original.id} />,
        },
      ]}
    />
  );
}

function CpuCell({ id }: { id: string }) {
  const stats = useServerStats(id);
  const cpu = stats?.cpu_perc ?? 0;
  const { cpuWarning: warning, cpuCritical: critical } =
    useServerThresholds(id);
  const intent: "Good" | "Warning" | "Critical" =
    cpu < warning ? "Good" : cpu < critical ? "Warning" : "Critical";
  return <StatCell value={stats ? cpu : undefined} intent={intent} />;
}

function MemCell({ id }: { id: string }) {
  const stats = useServerStats(id);
  const used = stats?.mem_used_gb ?? 0;
  const total = stats?.mem_total_gb ?? 0;
  const perc = total > 0 ? (used / total) * 100 : 0;
  const { memWarning: warning, memCritical: critical } =
    useServerThresholds(id);
  const intent: "Good" | "Warning" | "Critical" =
    perc < warning ? "Good" : perc < critical ? "Warning" : "Critical";
  return <StatCell value={stats ? perc : undefined} intent={intent} />;
}

function DiskCell({ id }: { id: string }) {
  const stats = useServerStats(id);
  const used = stats?.disks?.reduce((acc, d) => acc + (d.used_gb || 0), 0) ?? 0;
  const total =
    stats?.disks?.reduce((acc, d) => acc + (d.total_gb || 0), 0) ?? 0;
  const perc = total > 0 ? (used / total) * 100 : 0;
  const { diskWarning: warning, diskCritical: critical } =
    useServerThresholds(id);
  const intent: "Good" | "Warning" | "Critical" =
    perc < warning ? "Good" : perc < critical ? "Warning" : "Critical";
  return (
    <StatCell
      value={stats ? perc : undefined}
      intent={intent}
      infoDisabled={!stats}
      info={
        <Stack gap="sm">
          {stats?.disks.map((disk) => (
            <Group
              key={disk.mount}
              justify="space-between"
              className="bordered-light"
              p="sm"
              bdrs="sm"
            >
              <Group>
                <Text c="dimmed">Mount:</Text>{" "}
                <Text ff="monospace" bg="accent.6" px="xs" bdrs="sm">
                  {disk.mount}
                </Text>
              </Group>
              <Text>-</Text>
              <Group gap="0.4rem">
                {disk.used_gb.toFixed(1)} GB <Text c="dimmed">of</Text>{" "}
                {disk.total_gb.toFixed(1)} GB <Text c="dimmed">in use</Text> (
                {((100 * disk.used_gb) / disk.total_gb).toFixed(1)} %)
              </Group>
            </Group>
          ))}
        </Stack>
      }
    />
  );
}

function LoadAvgCell({ id }: { id: string }) {
  const stats = useServerStats(id);
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

function NetCell({ id }: { id: string }) {
  const stats = useServerStats(id);
  const ingress = stats?.network_ingress_bytes ?? 0;
  const egress = stats?.network_egress_bytes ?? 0;
  if (!stats) {
    return <Text c="dimmed">N/A</Text>;
  }
  return <Text>{fmtRateBytes(ingress + egress)}</Text>;
}
