import { TableTags } from "@components/tags";
import { DataTable, SortableHeader } from "@ui/data-table";
import { fmt_version } from "@lib/formatting";
import { ResourceLink } from "../common";
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
          accessorKey: "name",
          header: ({ column }) => (
            <SortableHeader column={column} title="Name" />
          ),
          cell: ({ row }) => <ResourceLink type="Build" id={row.original.id} />,
          size: 200,
        },
        {
          // header: ({ column }) => (
          //   <SortableHeader column={column} title="Source" />
          // ),
          header: "Source",
          accessorFn: ({ info: { files_on_host, repo } }) => {
            if (files_on_host) {
              return "Files on Server";
            } else if (repo) {
              return repo;
            } else {
              return "UI Defined";
            }
          },
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
