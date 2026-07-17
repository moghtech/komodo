import { useResourceSelectionState } from "@/lib/hooks";
import { DataTable, SortableHeader } from "mogh_ui";
import { Group, BoxProps } from "@mantine/core";
import { Types } from "komodo_client";
import ResourceLink from "@/resources/link";
import { StackComponents } from ".";
import TableTags from "@/components/tags/table";
import FileSource from "@/components/file-source";
import StackUpdateAvailable from "./update-available";

const SORT_KEYS = ["Name", "Source", "Host", "State"];

export default function StackTable({
  resources,
  onServerSort,
  ...boxProps
}: {
  resources: Types.StackListItem[];
  /** When provided, sorting is handled server side,
   * and sort updates are passed to this callback. */
  onServerSort?: (sort: {
    sort_by?: string;
    sort_desc?: boolean;
  }) => void;
} & BoxProps) {
  const selectionState = useResourceSelectionState("Stack");

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
      tableKey="stack-table"
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
          cell: ({ row }) => {
            return (
              <Group wrap="nowrap">
                <ResourceLink type="Stack" id={row.original.id} />
                <StackUpdateAvailable id={row.original.id} small />
              </Group>
            );
          },
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
            <SortableHeader column={column} title="Host" />
          ),
          id: "Host",
          accessorKey: "info.server_id",
          sortingFn: (a, b) => {
            const name_a = a.original.info.swarm_id
              ? a.original.info.swarm_name
              : a.original.info.server_name;
            const name_b = b.original.info.swarm_id
              ? b.original.info.swarm_name
              : b.original.info.server_name;

            if (!name_a && !name_b) return 0;
            if (!name_a) return 1;
            if (!name_b) return -1;

            if (name_a > name_b) return 1;
            else if (name_a < name_b) return -1;
            else return 0;
          },
          cell: ({ row }) =>
            row.original.info.swarm_id ? (
              <ResourceLink type="Swarm" id={row.original.info.swarm_id} />
            ) : (
              <ResourceLink type="Server" id={row.original.info.server_id} />
            ),
          size: 200,
        },
        {
          id: "State",
          accessorKey: "info.state",
          header: ({ column }) => (
            <SortableHeader column={column} title="State" />
          ),
          cell: ({ row }) => <StackComponents.State id={row.original.id} />,
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
