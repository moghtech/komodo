import { TableTags } from "@components/tags";
import { useRead, useSelectedResources } from "@lib/hooks";
import { DataTable, SortableHeader } from "@ui/data-table";
import { ServerComponents } from ".";
import { ResourceLink } from "../common";
import { Types } from "komodo_client";
import { useCallback } from "react";

export const ServerTable = ({
  servers,
}: {
  servers: Types.ServerListItem[];
}) => {
  const [_, setSelectedResources] = useSelectedResources("Server");
  const deployments = useRead("ListDeployments", {}).data;
  const stacks = useRead("ListStacks", {}).data;
  const repos = useRead("ListRepos", {}).data;
  const resourcesCount = useCallback(
    (id: string) => {
      return (
        (deployments?.filter((d) => d.info.server_id === id).length || 0) +
        (stacks?.filter((d) => d.info.server_id === id).length || 0) +
        (repos?.filter((d) => d.info.server_id === id).length || 0)
      );
    },
    [deployments, stacks, repos]
  );
  return (
    <DataTable
      tableKey="servers"
      data={servers}
      selectOptions={{
        selectKey: ({ name }) => name,
        onSelect: setSelectedResources,
      }}
      columns={[
        {
          accessorKey: "name",
          header: ({ column }) => (
            <SortableHeader column={column} title="Name" />
          ),
          cell: ({ row }) => (
            <ResourceLink type="Server" id={row.original.id} />
          ),
          size: 200,
        },
        {
          accessorKey: "id",
          sortingFn: (a, b) => {
            const sa = resourcesCount(a.original.id);
            const sb = resourcesCount(b.original.id);

            if (!sa && !sb) return 0;
            if (!sa) return 1;
            if (!sb) return -1;

            if (sa > sb) return 1;
            else if (sa < sb) return -1;
            else return 0;
          },
          header: ({ column }) => (
            <SortableHeader column={column} title="Resources" />
          ),
          cell: ({ row }) => {
            return <>{resourcesCount(row.original.id)}</>;
          },
        },
        {
          accessorKey: "info.region",
          header: ({ column }) => (
            <SortableHeader column={column} title="Region" />
          ),
        },
        {
          accessorKey: "info.state",
          header: ({ column }) => (
            <SortableHeader column={column} title="State" />
          ),
          cell: ({ row }) => <ServerComponents.State id={row.original.id} />,
        },
        {
          header: "Tags",
          cell: ({ row }) => <TableTags tag_ids={row.original.tags} />,
        },
      ]}
    />
  );
};
