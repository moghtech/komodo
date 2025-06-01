import { DataTable, SortableHeader } from "@ui/data-table";
import { ResourceLink, StandardSource } from "../common";
import { TableTags } from "@components/tags";
import { Types } from "komodo_client";
import { ResourceSyncComponents } from ".";
import { useSelectedResources } from "@lib/hooks";

export const ResourceSyncTable = ({
  syncs,
}: {
  syncs: Types.ResourceSyncListItem[];
}) => {
  const [_, setSelectedResources] = useSelectedResources("ResourceSync");
  return (
    <DataTable
      tableKey="syncs"
      data={syncs}
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
        {
          header: ({ column }) => (
            <SortableHeader column={column} title="Repo" />
          ),
          accessorKey: "info.repo",
          cell: ({ row }) => <StandardSource info={row.original.info} />,
          size: 200,
        },
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
};
