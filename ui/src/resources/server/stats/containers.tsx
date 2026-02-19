import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { DataTable, SortableHeader } from "@/ui/data-table";
import Section from "@/ui/section";
import ShowHideButton from "@/ui/show-hide-button";
import { Group, Text, TextInput } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { useState } from "react";
import { filterBySplit } from "@/lib/utils";
import DockerResourceLink from "@/components/docker/link";
import { useIsServerAvailable } from "../hooks";

export default function ServerContainerStats({ id }: { id: string }) {
  const [search, setSearch] = useState("");
  const [show, setShow] = useLocalStorage({
    key: "server-stats-containers-show-v2",
    defaultValue: true,
  });
  const isServerAvailable = useIsServerAvailable(id);
  const containers = useRead(
    "ListDockerContainers",
    {
      server: id,
    },
    {
      enabled: isServerAvailable && show,
    },
  ).data?.filter((c) => c.stats);
  const filtered = filterBySplit(
    containers,
    search,
    (container) => container.name,
  );
  return (
    <Section
      title="Containers"
      titleRight={
        <Group ml="xl" gap="md">
          <TextInput
            leftSection={<ICONS.Search size="1rem" />}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="search..."
            w={{ base: 200, lg: 300 }}
          />
          <ShowHideButton show={show} setShow={setShow} />
        </Group>
      }
    >
      {show && (
        <DataTable
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
      )}
    </Section>
  );
}
