import ContainerPorts from "@/components/docker/container-ports";
import DockerResourceLink from "@/components/docker/link";
import { containerStateIntention } from "@/lib/color";
import { useRead, useTagsFilter } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { DataTable, SortableHeader, useDebounce } from "mogh_ui";
import { Page } from "mogh_ui";
import { StatusBadge } from "mogh_ui";
import { Group, Stack } from "@mantine/core";
import { useEffect, useMemo, useState } from "react";
import { DividedChildren } from "mogh_ui";
import ResourceLink from "@/resources/link";
import { SearchInput } from "mogh_ui";
import TagsFilter from "@/components/tags/filter";
import ResourceMultiSelector from "@/resources/multi-selector";
import ListPagination, {
  RESOURCE_PAGE_SIZE,
} from "@/components/list-pagination";

export default function Containers() {
  const [selectedServers, setSelectedServers] = useState<string[]>([]);

  const [page, setPage] = useState(0);
  const [search, setSearch] = useState("");
  const debouncedSearch = useDebounce(search, 200);
  const searchTerms = useMemo(() => {
    // back to first page if search changes
    setPage(0);
    return debouncedSearch
      .toLowerCase()
      .split(" ")
      .map((term) => term.trim())
      .filter((term) => term);
  }, [debouncedSearch]);

  const tags = useTagsFilter();

  // Set to page 0 whenever any filter changes,
  // otherwise the query can point past the last page and come back empty.
  useEffect(() => {
    setPage(0);
  }, [selectedServers, tags]);

  const _containers =
    useRead(
      "ListAllContainers",
      {
        terms: searchTerms,
        servers: selectedServers,
        tags,
        page,
        limit: RESOURCE_PAGE_SIZE,
      },
      { refetchInterval: 15_000 },
    ).data ?? [];

  // Prevent flashes during loading
  const containers = useDebounce(_containers, 300);

  const Table = useMemo(
    () => (
      <DataTable
        data={containers}
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
              const sa = a.original.server_name;
              const sb = b.original.server_name;

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
    ),
    [containers],
  );

  return (
    <Page
      title="Containers"
      icon={ICONS.Container}
      description="See all containers across all servers."
    >
      <Stack>
        <Group justify="space-between">
          <Group w={{ base: "100%", xs: "fit-content" }}>
            <ResourceMultiSelector
              type="Server"
              value={selectedServers}
              onChange={setSelectedServers}
            />
            <ListPagination
              page={page}
              setPage={setPage}
              count={containers.length}
            />
          </Group>

          <Group w={{ base: "100%", xs: "fit-content" }}>
            <TagsFilter />
            <SearchInput value={search} onSearch={setSearch} />
          </Group>
        </Group>

        {Table}
      </Stack>
    </Page>
  );
}
