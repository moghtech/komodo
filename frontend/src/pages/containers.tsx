import { Page } from "@components/layouts";
import { ResourceLink } from "@components/resources/common";
import {
  ContainerPortsTableView,
  DockerResourceLink,
  StatusBadge,
} from "@components/util";
import { container_state_intention } from "@lib/color";
import { useRead } from "@lib/hooks";
import { DataTable, SortableHeader } from "@ui/data-table";
import { Input } from "@ui/input";
import { Box, Search } from "lucide-react";
import { Fragment, useCallback, useMemo, useState } from "react";

export default function ContainersPage() {
  const [search, setSearch] = useState("");
  const searchSplit = search
    .toLowerCase()
    .split(" ")
    .filter((term) => term);
  const servers = useRead("ListServers", {}).data;
  const serverName = useCallback(
    (id: string) => servers?.find((server) => server.id === id)?.name,
    [servers]
  );
  const _containers = useRead("ListAllDockerContainers", {}).data;
  const containers = useMemo(
    () =>
      _containers?.filter((c) => {
        if (searchSplit.length === 0) return true;
        const lower = c.name.toLowerCase();
        return searchSplit.every((search) => lower.includes(search));
      }),
    [_containers, searchSplit]
  );
  return (
    <Page
      title="Containers"
      subtitle={
        <div className="text-muted-foreground">
          See all containers across all servers
        </div>
      }
      icon={<Box className="w-8 h-8" />}
    >
      <div className="flex flex-col gap-4">
        <div className="flex items-center justify-between">
          <div></div>
          <div className="relative">
            <Search className="w-4 absolute top-[50%] left-3 -translate-y-[50%] text-muted-foreground" />
            <Input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="search..."
              className="pl-8 w-[200px] lg:w-[300px]"
            />
          </div>
        </div>
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
                  type="container"
                  server_id={row.original.server_id!}
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
              accessorKey: "image",
              size: 300,
              header: ({ column }) => (
                <SortableHeader column={column} title="Image" />
              ),
              cell: ({ row }) => (
                <DockerResourceLink
                  type="image"
                  server_id={row.original.server_id!}
                  name={row.original.image}
                  id={row.original.image_id}
                />
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
                    intent={container_state_intention(state)}
                  />
                );
              },
            },
            {
              accessorKey: "networks.0",
              size: 200,
              header: ({ column }) => (
                <SortableHeader column={column} title="Networks" />
              ),
              cell: ({ row }) =>
                (row.original.networks?.length ?? 0) > 0 ? (
                  <div className="flex items-center gap-x-2 flex-wrap">
                    {row.original.networks?.map((network, i) => (
                      <Fragment key={network}>
                        <DockerResourceLink
                          type="network"
                          server_id={row.original.server_id!}
                          name={network}
                        />
                        {i !== row.original.networks!.length - 1 && (
                          <div className="text-muted-foreground">|</div>
                        )}
                      </Fragment>
                    ))}
                  </div>
                ) : (
                  row.original.network_mode && (
                    <DockerResourceLink
                      type="network"
                      server_id={row.original.server_id!}
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
                <ContainerPortsTableView
                  ports={row.original.ports ?? []}
                  server_id={row.original.server_id}
                />
              ),
            },
            // {
            //   accessorKey: "volumes.0",
            //   minSize: 300,
            //   header: ({ column }) => (
            //     <SortableHeader column={column} title="Volumes" />
            //   ),
            //   cell: ({ row }) => (
            //     <div className="flex items-center gap-x-2 flex-wrap">
            //       {row.original.volumes.map((volume, i) => (
            //         <Fragment key={volume}>
            //           <DockerResourceLink
            //             type="volume"
            //             server_id={row.original.server_id!}
            //             name={volume}
            //           />
            //           {i !== row.original.volumes.length - 1 && (
            //             <div className="text-muted-foreground">|</div>
            //           )}
            //         </Fragment>
            //       ))}
            //     </div>
            //   ),
            // },
          ]}
        />
      </div>
    </Page>
  );
}
