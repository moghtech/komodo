import { useSelectedResources } from "@/lib/hooks";
import { DataTable, SortableHeader } from "@/ui/data-table";
import { Types } from "komodo_client";
import { ResourceLink } from "../common";
import { fmtVersion } from "@/lib/formatting";
import { BuildComponents } from ".";
import TableTags from "@/components/tags/table";
import { TableProps } from "@mantine/core";

export default function BuildTable({
  resources,
  ...tableProps
}: {
  resources: Types.BuildListItem[];
} & TableProps) {
  const [_, setSelectedResources] = useSelectedResources("Build");

  return (
    <DataTable
      {...tableProps}
      tableKey="builds"
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
          cell: ({ row }) => <ResourceLink type="Build" id={row.original.id} />,
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
          header: "Version",
          accessorFn: ({ info }) => fmtVersion(info.version),
          size: 120,
        },
        {
          accessorKey: "info.state",
          header: ({ column }) => (
            <SortableHeader column={column} title="State" />
          ),
          cell: ({ row }) => <BuildComponents.State id={row.original.id} />,
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
