import { useSelectedResources } from "@/lib/hooks";
import { DataTable, SortableHeader } from "mogh_ui";
import { Types } from "komodo_client";
import { fmtVersion } from "@/lib/formatting";
import { BuildComponents } from ".";
import TableTags from "@/components/tags/table";
import { BoxProps } from "@mantine/core";
import ResourceLink from "@/resources/link";
import FileSource from "@/components/file-source";

export default function BuildTable({
  resources,
  ...boxProps
}: {
  resources: Types.BuildListItem[];
} & BoxProps) {
  const [_, setSelectedResources] = useSelectedResources("Build");

  return (
    <DataTable
      {...boxProps}
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
        {
          header: ({ column }) => (
            <SortableHeader column={column} title="Source" />
          ),
          accessorKey: "info.repo",
          cell: ({ row }) => <FileSource info={row.original.info} />,
          size: 200,
        },
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
          cell: ({ row }) => <TableTags tagIds={row.original.tags} />,
        },
      ]}
    />
  );
}
