import { Types } from "komodo_client";
import { useSelectedResources } from "@/lib/hooks";
import { DataTable, SortableHeader } from "mogh_ui";
import ResourceLink from "@/resources/link";
import { RepoComponents } from ".";
import TableTags from "@/components/tags/table";
import { BoxProps } from "@mantine/core";
import RepoLink from "@/components/repo-link";

const SORT_KEYS = ["Name", "Repo", "Branch", "State"];

export default function RepoTable({
  resources,
  onServerSort,
  ...boxProps
}: {
  resources: Types.RepoListItem[];
  /** When provided, sorting is handled server side,
   * and sort updates are passed to this callback. */
  onServerSort?: (sort: {
    sort_by?: string;
    sort_desc?: boolean;
  }) => void;
} & BoxProps) {
  const [_, setSelectedResources] = useSelectedResources("Repo");

  return (
    <DataTable
      {...boxProps}
      manualSorting={!!onServerSort}
      onSortingStateChange={
        onServerSort &&
        ((sorting) => {
          const sort = sorting.find((s) => SORT_KEYS.includes(s.id));
          onServerSort(
            sort ? { sort_by: sort.id, sort_desc: sort.desc } : {},
          );
        })
      }
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
          id: "Name",
          accessorKey: "name",
          cell: ({ row }) => <ResourceLink type="Repo" id={row.original.id} />,
          size: 200,
        },
        {
          header: ({ column }) => (
            <SortableHeader column={column} title="Repo" />
          ),
          id: "Repo",
          accessorKey: "info.repo",
          cell: ({ row }) => (
            <RepoLink
              repo={row.original.info.repo}
              link={row.original.info.repo_link}
            />
          ),
          size: 200,
        },
        {
          header: ({ column }) => (
            <SortableHeader column={column} title="Branch" />
          ),
          id: "Branch",
          accessorKey: "info.branch",
          size: 200,
        },
        {
          header: ({ column }) => (
            <SortableHeader column={column} title="State" />
          ),
          id: "State",
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
