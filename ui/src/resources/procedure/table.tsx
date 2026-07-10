import { useSelectedResources } from "@/lib/hooks";
import { DataTable, SortableHeader } from "mogh_ui";
import { Types } from "komodo_client";
import ResourceLink from "@/resources/link";
import { ProcedureComponents } from ".";
import TableTags from "@/components/tags/table";
import { BoxProps } from "@mantine/core";

const SORT_KEYS = ["Name", "State", "NextRun"];

export default function ProcedureTable({
  resources,
  onServerSort,
  ...boxProps
}: {
  resources: Types.ProcedureListItem[];
  /** When provided, sorting is handled server side,
   * and sort updates are passed to this callback. */
  onServerSort?: (sort: {
    sort_by?: string;
    sort_desc?: boolean;
  }) => void;
} & BoxProps) {
  const [_, setSelectedResources] = useSelectedResources("Procedure");

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
      tableKey="procedures"
      data={resources}
      selectOptions={{
        selectKey: ({ name }) => name,
        onSelect: setSelectedResources,
      }}
      columns={[
        {
          id: "Name",
          accessorKey: "name",
          header: ({ column }) => (
            <SortableHeader column={column} title="Name" />
          ),
          cell: ({ row }) => (
            <ResourceLink type="Procedure" id={row.original.id} />
          ),
        },
        {
          id: "State",
          accessorKey: "info.state",
          header: ({ column }) => (
            <SortableHeader column={column} title="State" />
          ),
          cell: ({ row }) => <ProcedureComponents.State id={row.original.id} />,
        },
        {
          id: "NextRun",
          accessorKey: "info.next_scheduled_run",
          header: ({ column }) => (
            <SortableHeader column={column} title="Next Run" />
          ),
          sortingFn: (a, b) => {
            const sa = a.original.info.next_scheduled_run;
            const sb = b.original.info.next_scheduled_run;

            if (!sa && !sb) return 0;
            if (!sa) return 1;
            if (!sb) return -1;

            if (sa > sb) return 1;
            else if (sa < sb) return -1;
            else return 0;
          },
          cell: ({ row }) =>
            row.original.info.next_scheduled_run
              ? new Date(row.original.info.next_scheduled_run).toLocaleString()
              : "Not Scheduled",
        },
        {
          header: "Tags",
          cell: ({ row }) => <TableTags tagIds={row.original.tags} />,
        },
      ]}
    />
  );
}
