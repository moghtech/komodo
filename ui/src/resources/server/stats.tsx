import { ReactNode, useState } from "react";
import { useStatsGranularity } from "./hooks";
import { usePermissions, useRead } from "@/lib/hooks";
import { useServer } from ".";
import { Types } from "komodo_client";
import { useLocalStorage } from "@mantine/hooks";
import { filterBySplit } from "@/lib/utils";
import Section from "@/ui/section";
import { DataTable, SortableHeader } from "@/ui/data-table";
import { Group, Text, TextInput } from "@mantine/core";
import { ICONS } from "@/theme/icons";
import ShowHideButton from "@/ui/show-hide-button";
import DockerResourceLink from "@/components/docker/link";

export default function ServerStats({
  id,
  titleOther,
}: {
  id: string;
  titleOther?: ReactNode;
}) {
  const [interval, setInterval] = useStatsGranularity();

  const { specific } = usePermissions({ type: "Server", id });
  const isServerAvailable = useServer(id)?.info.state === Types.ServerState.Ok;

  const stats = useRead(
    "GetSystemStats",
    { server: id },
    {
      enabled: isServerAvailable,
      refetchInterval: 10_000,
    },
  ).data;
  const info = useRead(
    "GetSystemInformation",
    { server: id },
    { enabled: isServerAvailable },
  ).data;

  // Get all the containers with stats
  const containers = useRead(
    "ListDockerContainers",
    {
      server: id,
    },
    {
      enabled: isServerAvailable,
    },
  ).data?.filter((c) => c.stats);
  const [show, setShow] = useLocalStorage({
    key: "server-stats-show-v2",
    defaultValue: {
      containers: true,
      disks: true,
    },
  });
  const [containerSearch, setContainerSearch] = useState("");
  const filteredContainers = filterBySplit(
    containers,
    containerSearch,
    (container) => container.name,
  );
  const diskUsed = stats?.disks.reduce((acc, curr) => (acc += curr.used_gb), 0);
  const diskTotal = stats?.disks.reduce(
    (acc, curr) => (acc += curr.total_gb),
    0,
  );

  const [showHistorical, setShowHistorical] = useState(false);

  return (
    <Section titleOther={titleOther}>
      <Section title="System Info">
        <DataTable
          tableKey="system-info"
          data={
            info
              ? [
                  {
                    ...info,
                    memTotal: stats?.mem_total_gb,
                    diskTotal,
                  },
                ]
              : []
          }
          columns={[
            {
              header: "Hostname",
              accessorKey: "host_name",
            },
            {
              header: "Os",
              accessorKey: "os",
            },
            {
              header: "Kernel",
              accessorKey: "kernel",
            },
            {
              header: "CPU",
              accessorKey: "cpu_brand",
            },
            {
              header: "Core Count",
              accessorFn: ({ core_count }) =>
                `${core_count} Core${(core_count || 0) > 1 ? "s" : ""}`,
            },
            {
              header: "Total Memory",
              accessorFn: ({ memTotal }) => `${memTotal?.toFixed(2)} GB`,
            },
            {
              header: "Total Disk Size",
              accessorFn: ({ diskTotal }) => `${diskTotal?.toFixed(2)} GB`,
            },
          ]}
        />
      </Section>

      <Section title="Current">{/* CURRENT */}</Section>

      <Section
        title="Containers"
        actions={
          <Group gap="md">
            <TextInput
              leftSection={<ICONS.Search size="1rem" />}
              value={containerSearch}
              onChange={(e) => setContainerSearch(e.target.value)}
              placeholder="search..."
              w={{ base: 200, lg: 300 }}
            />
            <ShowHideButton
              show={show.containers}
              setShow={(containers) =>
                setShow((show) => ({ ...show, containers }))
              }
            />
          </Group>
        }
      >
        {show.containers && (
          <DataTable
            tableKey="container-stats"
            data={filteredContainers}
            columns={[
              {
                accessorKey: "name",
                size: 200,
                header: ({ column }) => (
                  <SortableHeader column={column} title="Name" />
                ),
                cell: ({ row }) => (
                  <DockerResourceLink
                    type="Container"
                    serverId={id}
                    name={row.original.name}
                  />
                ),
              },
              {
                accessorKey: "stats.cpu_perc",
                size: 100,
                header: ({ column }) => (
                  <SortableHeader column={column} title="CPU" />
                ),
              },
              {
                accessorKey: "stats.mem_perc",
                size: 200,
                header: ({ column }) => (
                  <SortableHeader column={column} title="Memory" />
                ),
                cell: ({ row }) => (
                  <Group gap="xs">
                    <Text>{row.original.stats?.mem_perc}</Text>
                    <Text c="muted" size="sm">({row.original.stats?.mem_usage})</Text>
                  </Group>
                ),
              },
              {
                accessorKey: "stats.net_io",
                size: 150,
                header: ({ column }) => (
                  <SortableHeader column={column} title="Net I/O" />
                ),
              },
              {
                accessorKey: "stats.block_io",
                size: 150,
                header: ({ column }) => (
                  <SortableHeader column={column} title="Block I/O" />
                ),
              },
              {
                accessorKey: "stats.pids",
                size: 100,
                header: ({ column }) => (
                  <SortableHeader column={column} title="PIDs" />
                ),
              },
            ]}
          />
        )}
      </Section>

      <Section
        title="Disks"
        actions={
          <Group gap="md">
            <Group gap="xs">
              <Text c="dimmed">Used:</Text>
              <Text>{diskUsed?.toFixed(2)} GB</Text>
            </Group>
            <Group gap="xs">
              <Text c="dimmed">Total:</Text>
              <Text>{diskTotal?.toFixed(2)} GB</Text>
            </Group>
            <ShowHideButton
              show={show.disks}
              setShow={(disks) => setShow((show) => ({ ...show, disks }))}
            />
          </Group>
        }
      >
        {show.disks && (
          <DataTable
            sortDescFirst
            tableKey="server-disks"
            data={
              stats?.disks.map((disk) => ({
                ...disk,
                percentage: 100 * (disk.used_gb / disk.total_gb),
              })) ?? []
            }
            columns={[
              {
                header: "Path",
                cell: ({ row }) => (
                  <div className="overflow-hidden overflow-ellipsis">
                    {row.original.mount}
                  </div>
                ),
              },
              {
                accessorKey: "used_gb",
                header: ({ column }) => (
                  <SortableHeader column={column} title="Used" sortDescFirst />
                ),
                cell: ({ row }) => <>{row.original.used_gb.toFixed(2)} GB</>,
              },
              {
                accessorKey: "total_gb",
                header: ({ column }) => (
                  <SortableHeader column={column} title="Total" sortDescFirst />
                ),
                cell: ({ row }) => <>{row.original.total_gb.toFixed(2)} GB</>,
              },
              {
                accessorKey: "percentage",
                header: ({ column }) => (
                  <SortableHeader
                    column={column}
                    title="Percentage"
                    sortDescFirst
                  />
                ),
                cell: ({ row }) => (
                  <>{row.original.percentage.toFixed(2)}% Full</>
                ),
              },
            ]}
          />
        )}
      </Section>

      {/* {specific.includes(Types.SpecificPermission.Processes) && (
        <Processes id={id} />
      )} */}

      <Section title="Historical">{/* CHARTS */}</Section>
    </Section>
  );
}
