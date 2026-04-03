import ContainerPorts from "@/components/docker/container-ports";
import DockerResourceLink from "@/components/docker/link";
import { containerStateIntention } from "@/lib/color";
import { useDebounce, useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { filterBySplit } from "@/lib/utils";
import { DataTable, SortableHeader } from "mogh_ui";
import { Page } from "mogh_ui";
import { StatusBadge } from "mogh_ui";
import { Group, MultiSelect, Stack } from "@mantine/core";
import { useCallback, useMemo, useState } from "react";
import { DividedChildren } from "mogh_ui";
import ResourceLink from "@/resources/link";
import { SearchInput } from "mogh_ui";

export default function Containers() {
  const [search, setSearch] = useState("");
  const [selectedServers, setSelectedServers] = useState<string[]>([]);

  const debouncedSearch = useDebounce(search, 300);

  const servers = useRead("ListServers", {}).data;
  const serverOptions = useMemo(
    () =>
      servers?.map((server) => ({
        label: server.name,
        value: server.id,
      })) || [],
    [servers],
  );

  const serverName = useCallback(
    (id: string) => servers?.find((server) => server.id === id)?.name,
    [servers],
  );

  const _containers = useRead("ListAllDockerContainers", {}).data?.filter(
    (container) =>
      !selectedServers.length ||
      (container.server_id && selectedServers.includes(container.server_id)),
  );

  const containers = useMemo(
    () => filterBySplit(_containers, debouncedSearch, (item) => item.name),
    [_containers, debouncedSearch],
  );

  return (
    <Page
      title="Containers"
      icon={ICONS.Container}
      description="See all containers across all servers."
    >
      <Stack>
        <Group justify="space-between">
          <MultiSelect
            placeholder="Filter by Servers"
            value={selectedServers}
            onChange={setSelectedServers}
            data={serverOptions}
            searchable
            clearable
          />
          <SearchInput value={search} onSearch={setSearch} />
        </Group>

        <DataTable
          data={containers ?? []}
          tableKey="containers-page-v1"
          columns={[
            {
              accessorKey: "name",
              size: 260,
              header: ({ column }) => (
                <SortableHeader column={column} title="Name" />
              ),
              cell: ({ row }) => (
                <DockerResourceLink
                  type="Container"
                  serverId={row.original.server_id!}
                  name={row.original.name}
                />
              ),
            },
            {
              accessorKey: "server_id",
              size: 200,
              sortingFn: (a, b) => {
                const sa = serverName(a.original.server_id!);
                const sb = serverName(b.original.server_id!);

                if (!sa && !sb) return 0;
                if (!sa) return -1;
                if (!sb) return 1;

                if (sa > sb) return 1;
                else if (sa < sb) return -1;
                else return 0;
              },
              header: ({ column }) => (
                <SortableHeader column={column} title="Server" />
              ),
              cell: ({ row }) => (
                <ResourceLink type="Server" id={row.original.server_id!} />
              ),
            },
            {
              accessorKey: "state",
              size: 160,
              header: ({ column }) => (
                <SortableHeader column={column} title="State" />
              ),
              cell: ({ row }) => {
                const state = row.original?.state;
                return (
                  <StatusBadge
                    text={state}
                    intent={containerStateIntention(state)}
                  />
                );
              },
            },
            {
              accessorKey: "image",
              size: 300,
              header: ({ column }) => (
                <SortableHeader column={column} title="Image" />
              ),
              cell: ({ row }) => (
                <DockerResourceLink
                  type="Image"
                  serverId={row.original.server_id!}
                  name={row.original.image}
                  id={row.original.image_id}
                />
              ),
            },
            {
              accessorKey: "networks.0",
              size: 200,
              header: ({ column }) => (
                <SortableHeader column={column} title="Networks" />
              ),
              cell: ({ row }) =>
                (row.original.networks?.length ?? 0) > 0 ? (
                  <DividedChildren>
                    {row.original.networks?.map((network) => (
                      <DockerResourceLink
                        key={network}
                        type="Network"
                        serverId={row.original.server_id!}
                        name={network}
                      />
                    ))}
                  </DividedChildren>
                ) : (
                  row.original.network_mode && (
                    <DockerResourceLink
                      type="Network"
                      serverId={row.original.server_id!}
                      name={row.original.network_mode}
                    />
                  )
                ),
            },
            {
              accessorKey: "ports.0",
              size: 200,
              sortingFn: (a, b) => {
                const getMinHostPort = (row: typeof a) => {
                  const ports = row.original.ports ?? [];
                  if (!ports.length) return Number.POSITIVE_INFINITY;
                  const nums = ports
                    .map((p) => p.PublicPort)
                    .filter((p): p is number => typeof p === "number")
                    .map((n) => Number(n));
                  if (!nums.length || nums.some((n) => Number.isNaN(n))) {
                    return Number.POSITIVE_INFINITY;
                  }
                  return Math.min(...nums);
                };
                const pa = getMinHostPort(a);
                const pb = getMinHostPort(b);
                return pa === pb ? 0 : pa > pb ? 1 : -1;
              },
              header: ({ column }) => (
                <SortableHeader column={column} title="Ports" />
              ),
              cell: ({ row }) => (
                <ContainerPorts
                  ports={row.original.ports ?? []}
                  serverId={row.original.server_id}
                />
              ),
            },
            {
              accessorKey: "volumes.0",
              size: 200,
              header: ({ column }) => (
                <SortableHeader column={column} title="Volumes" />
              ),
              cell: ({ row }) => (
                <DividedChildren>
                  {row.original.volumes?.map((volume) => (
                    <DockerResourceLink
                      key={volume}
                      type="Volume"
                      serverId={row.original.server_id!}
                      name={volume}
                    />
                  ))}
                </DividedChildren>
              ),
            },
          ]}
        />
      </Stack>
    </Page>
  );
}
