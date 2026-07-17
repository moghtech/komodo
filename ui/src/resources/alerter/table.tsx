import { useSelectedResources } from "@/lib/hooks";
import { DataTable, SortableHeader } from "mogh_ui";
import { Badge, BoxProps } from "@mantine/core";
import { Types } from "komodo_client";
import ResourceLink from "@/resources/link";
import TableTags from "@/components/tags/table";
import { StatusBadge } from "mogh_ui";
import { RowSelectionState } from "@tanstack/react-table";
import { setSelectedStateHandler } from "@/lib/utils";

const SORT_KEYS = ["Name", "Type", "Enabled"];

export default function AlerterTable({
  resources,
  onServerSort,
  ...boxProps
}: {
  resources: Types.AlerterListItem[];
  /** When provided, sorting is handled server side,
   * and sort updates are passed to this callback. */
  onServerSort?: (sort: {
    sort_by?: string;
    sort_desc?: boolean;
  }) => void;
} & BoxProps) {
  const [selectedResources, setSelectedResources] =
    useSelectedResources("Alerter");
  const selectionState = selectedResources.reduce((state, item) => {
    state[item] = true;
    return state;
  }, {} as RowSelectionState);
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
      tableKey="alerters"
      data={resources}
      selectOptions={{
        selectKey: ({ name }) => name,
        state: [
          selectionState,
          (state) =>
            setSelectedStateHandler(
              state,
              selectionState,
              setSelectedResources,
            ),
        ],
      }}
      columns={[
        {
          id: "Name",
          accessorKey: "name",
          header: ({ column }) => (
            <SortableHeader column={column} title="Name" />
          ),
          cell: ({ row }) => (
            <ResourceLink type="Alerter" id={row.original.id} />
          ),
        },
        {
          id: "Type",
          accessorKey: "info.endpoint_type",
          header: ({ column }) => (
            <SortableHeader column={column} title="Type" />
          ),
          cell: ({ row }) => (
            <Badge size="lg">{row.original.info.endpoint_type}</Badge>
          ),
        },
        {
          id: "Enabled",
          accessorKey: "info.enabled",
          header: ({ column }) => (
            <SortableHeader column={column} title="Enabled" />
          ),
          cell: ({ row }) => (
            <StatusBadge
              text={row.original.info.enabled ? "Enabled" : "Disabled"}
              intent={row.original.info.enabled ? "Good" : "Critical"}
            />
          ),
        },
        {
          header: "Tags",
          cell: ({ row }) => <TableTags tagIds={row.original.tags} />,
        },
      ]}
    />
  );
}
