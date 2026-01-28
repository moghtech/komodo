import { useSelectedResources } from "@/lib/hooks";
import { ResourceLink } from "@/resources/common";
import { DataTable, SortableHeader } from "@/ui/data-table";
import { Types } from "komodo_client";
import { SwarmComponents } from "..";
import TableTags from "@/components/tags/table";
import { TableProps } from "@mantine/core";

export default function SwarmTable({
  resources,
  ...tableProps
}: {
  resources: Types.SwarmListItem[];
} & TableProps) {
  const [_, setSelectedResources] = useSelectedResources("Swarm");

  return (
    <DataTable
      {...tableProps}
      tableKey="swarm-table"
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
          cell: ({ row }) => <ResourceLink type="Swarm" id={row.original.id} />,
          size: 200,
        },
        {
          header: ({ column }) => (
            <SortableHeader column={column} title="State" />
          ),
          accessorKey: "info.state",
          cell: ({ row }) => <SwarmComponents.State id={row.original.id} />,
          size: 120,
        },
        {
          header: "Tags",
          cell: ({ row }) => <TableTags tag_ids={row.original.tags} />,
        },
      ]}
    />
  );
}
