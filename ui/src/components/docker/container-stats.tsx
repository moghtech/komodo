
import { DataTable, SortableHeader } from "@/ui/data-table";

import { Group, Text } from "@mantine/core";
import DockerResourceLink from "@/components/docker/link";
import { filterBySplit } from "@/lib/utils";
import { Types } from "komodo_client";

export interface ContainerStatsProps {
    containers: Types.ContainerListItem[];
    search: string;
}

export default function ContainerStats({containers, search}: ContainerStatsProps) {
    const filtered = filterBySplit(
        containers,
        search,
        (container) => container.name,
    );
    return (
        <DataTable
          sortDescFirst
          mah="min(400px, calc(100vh - 320px))"
          tableKey="container-stats"
          data={filtered}
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
                  serverId={row.original.server_id || ""}
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
                  <Text c="muted" size="sm">
                    ({row.original.stats?.mem_usage})
                  </Text>
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
  );
}