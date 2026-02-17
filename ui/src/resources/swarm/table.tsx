import { useSelectedResources } from "@/lib/hooks";
import ResourceLink from "@/resources/link";
import { DataTable, SortableHeader } from "@/ui/data-table";
import { Types } from "komodo_client";
import { SwarmComponents } from ".";
import TableTags from "@/components/tags/table";
import { BoxProps } from "@mantine/core";

export default function SwarmTable({
  resources,
  ...boxProps
}: {
  resources: Types.SwarmListItem[];
} & BoxProps) {
  const [_, setSelectedResources] = useSelectedResources("Swarm");

  return (
    <DataTable
      {...boxProps}
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
          cell: ({ row }) => <TableTags tagIds={row.original.tags} />,
        },
      ]}
    />
  );
}
