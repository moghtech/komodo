import { useSelectedResources } from "@/lib/hooks";
import { DataTable, SortableHeader } from "@/ui/data-table";
import { Types } from "komodo_client";
import { ResourceLink } from "../common";
import { ResourceSyncComponents } from ".";
import TableTags from "@/components/tags/table";
import { TableProps } from "@mantine/core";

export default function ResourceSyncTable({
  resources,
  ...tableProps
}: {
  resources: Types.ResourceSyncListItem[];
} & TableProps) {
  const [_, setSelectedResources] = useSelectedResources("ResourceSync");
  return (
    <DataTable
      {...tableProps}
      tableKey="syncs"
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
          cell: ({ row }) => (
            <ResourceLink type="ResourceSync" id={row.original.id} />
          ),
          size: 200,
        },
        // {
        //   header: ({ column }) => (
        //     <SortableHeader column={column} title="Repo" />
        //   ),
        //   accessorKey: "info.repo",
        //   cell: ({ row }) => <StandardSource info={row.original.info} />,
        //   size: 200,
        // },
        {
          header: ({ column }) => (
            <SortableHeader column={column} title="Branch" />
          ),
          accessorKey: "info.branch",
          size: 200,
        },
        {
          header: ({ column }) => (
            <SortableHeader column={column} title="State" />
          ),
          accessorKey: "info.state",
          cell: ({ row }) => (
            <ResourceSyncComponents.State id={row.original.id} />
          ),
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
