import { useSelectedResources } from "@/lib/hooks";
import { DataTable, SortableHeader } from "mogh_ui";
import { BoxProps } from "@mantine/core";
import { Types } from "komodo_client";
import ResourceLink from "@/resources/link";
import TableTags from "@/components/tags/table";
import BuilderInstanceType from "./instance-type";

export default function BuilderTable({
  resources,
  ...boxProps
}: {
  resources: Types.BuilderListItem[];
} & BoxProps) {
  const [_, setSelectedResources] = useSelectedResources("Builder");
  return (
    <DataTable
      {...boxProps}
      tableKey="builders"
      data={resources}
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
            <ResourceLink type="Builder" id={row.original.id} />
          ),
        },
        {
          accessorKey: "info.builder_type",
          header: ({ column }) => (
            <SortableHeader column={column} title="Provider" />
          ),
        },
        {
          accessorKey: "info.instance_type",
          header: ({ column }) => (
            <SortableHeader column={column} title="Instance Type" />
          ),
          cell: ({ row }) => <BuilderInstanceType id={row.original.id} />,
        },
        {
          header: "Tags",
          cell: ({ row }) => <TableTags tagIds={row.original.tags} />,
        },
      ]}
    />
  );
}
