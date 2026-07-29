import { useResourceSelectionState } from "@/lib/hooks";
import { DataTable, SortableHeader } from "mogh_ui";
import { Types } from "komodo_client";
import ResourceLink from "@/resources/link";
import { ResourceSyncComponents } from ".";
import TableTags from "@/components/tags/table";
import { BoxProps } from "@mantine/core";
import FileSource from "@/components/file-source";

const SORT_KEYS = ["Name", "Source", "Branch", "State"];

export default function ResourceSyncTable({
  resources,
  onServerSort,
  ...boxProps
}: {
  resources: Types.ResourceSyncListItem[];
  /** When provided, sorting is handled server side,
   * and sort updates are passed to this callback. */
  onServerSort?: (sort: {
    sort_by?: string;
    sort_desc?: boolean;
  }) => void;
} & BoxProps) {
  const selectionState = useResourceSelectionState("ResourceSync");
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
      tableKey="syncs"
      data={resources}
      selectOptions={{
        selectKey: ({ name }) => name,
        state: selectionState,
      }}
      columns={[
        {
          header: ({ column }) => (
            <SortableHeader column={column} title="Name" />
          ),
          id: "Name",
          accessorKey: "name",
          cell: ({ row }) => (
            <ResourceLink type="ResourceSync" id={row.original.id} />
          ),
          size: 200,
        },
        {
          header: ({ column }) => (
            <SortableHeader column={column} title="Source" />
          ),
          id: "Source",
          accessorKey: "info.repo",
          cell: ({ row }) => <FileSource info={row.original.info} />,
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
          cell: ({ row }) => (
            <ResourceSyncComponents.State id={row.original.id} />
          ),
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
