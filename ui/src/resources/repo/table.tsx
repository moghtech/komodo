import { Types } from "komodo_client";
import { useSelectedResources } from "@/lib/hooks";
import { DataTable, SortableHeader } from "@/ui/data-table";
import ResourceLink from "@/resources/link";
import { RepoComponents } from ".";
import TableTags from "@/components/tags/table";
import { BoxProps } from "@mantine/core";

export default function RepoTable({
  resources,
  ...boxProps
}: { resources: Types.RepoListItem[] } & BoxProps) {
  const [_, setSelectedResources] = useSelectedResources("Repo");

  return (
    <DataTable
      {...boxProps}
      tableKey="repo-table"
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
          cell: ({ row }) => <ResourceLink type="Repo" id={row.original.id} />,
          size: 200,
        },
        // {
        //   header: ({ column }) => (
        //     <SortableHeader column={column} title="Repo" />
        //   ),
        //   accessorKey: "info.repo",
        //   cell: ({ row }) => (
        //     <RepoLink
        //       repo={row.original.info.repo}
        //       link={row.original.info.repo_link}
        //     />
        //   ),
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
          cell: ({ row }) => <RepoComponents.State id={row.original.id} />,
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
