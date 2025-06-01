import { TableTags } from "@components/tags";
import { DataTable, SortableHeader } from "@ui/data-table";
import { fmt_version } from "@lib/formatting";
import { ResourceLink, StandardSource } from "../common";
import { BuildComponents } from ".";
import { Types } from "komodo_client";
import { useSelectedResources } from "@lib/hooks";

export const BuildTable = ({ builds }: { builds: Types.BuildListItem[] }) => {
  const [_, setSelectedResources] = useSelectedResources("Build");

  return (
    <DataTable
      tableKey="builds"
      data={builds}
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
          cell: ({ row }) => <StandardSource info={row.original.info} />,
          size: 200,
        },
        {
          header: "Version",
          accessorFn: ({ info }) => fmt_version(info.version),
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
};
