import { useResourceName, useSelectedResources } from "@/lib/hooks";
import { DataTable, SortableHeader } from "@/ui/data-table";
import { Group, BoxProps } from "@mantine/core";
import { Types } from "komodo_client";
import { ResourceLink } from "@/resources/common";
import { StackComponents } from ".";
import TableTags from "@/components/tags/table";

export default function StackTable({
  resources,
  ...boxProps
}: {
  resources: Types.StackListItem[];
} & BoxProps) {
  const swarmName = useResourceName("Swarm");
  const serverName = useResourceName("Server");

  const [_, setSelectedResources] = useSelectedResources("Stack");

  return (
    <DataTable
      {...boxProps}
      tableKey="stack-table"
      data={resources}
      selectOptions={{
        selectKey: ({ name }) => name,
        onSelect: setSelectedResources,
      }}
      columns={[
        {
          header: ({ column }) => (
            <SortableHeader column={column} title="Name" />
          ),
          accessorKey: "name",
          cell: ({ row }) => {
            return (
              <Group justify="space-between">
                <ResourceLink type="Stack" id={row.original.id} />
                {/* <UpdateAvailable id={row.original.id} small /> */}
              </Group>
            );
          },
          size: 200,
        },
        // {
        //   header: ({ column }) => (
        //     <SortableHeader column={column} title="Source" />
        //   ),
        //   accessorKey: "info.repo",
        //   cell: ({ row }) => <StandardSource info={row.original.info} />,
        //   size: 200,
        // },
        {
          header: ({ column }) => (
            <SortableHeader column={column} title="Host" />
          ),
          accessorKey: "info.server_id",
          sortingFn: (a, b) => {
            const name_a = a.original.info.swarm_id
              ? swarmName(a.original.info.swarm_id)
              : serverName(a.original.info.server_id);
            const name_b = b.original.info.swarm_id
              ? swarmName(b.original.info.swarm_id)
              : serverName(b.original.info.server_id);

            if (!name_a && !name_b) return 0;
            if (!name_a) return 1;
            if (!name_b) return -1;

            if (name_a > name_b) return 1;
            else if (name_a < name_b) return -1;
            else return 0;
          },
          cell: ({ row }) =>
            row.original.info.swarm_id ? (
              <ResourceLink type="Swarm" id={row.original.info.swarm_id} />
            ) : (
              <ResourceLink type="Server" id={row.original.info.server_id} />
            ),
          size: 200,
        },
        {
          accessorKey: "info.state",
          header: ({ column }) => (
            <SortableHeader column={column} title="State" />
          ),
          cell: ({ row }) => <StackComponents.State id={row.original.id} />,
          size: 120,
        },
        {
          header: "Tags",
          cell: ({ row }) => <TableTags tagIds={row.original.tags} />,
        },
      ]}
    />
  );
}
