import { useSelectedResources } from "@/lib/hooks";
import { DataTable, SortableHeader } from "mogh_ui";
import { BoxProps } from "@mantine/core";
import { Types } from "komodo_client";
import ResourceLink from "@/resources/link";
import TableTags from "@/components/tags/table";
import BuilderInstanceType from "./instance-type";
import { RowSelectionState } from "@tanstack/react-table";
import { setSelectedStateHandler } from "@/lib/utils";

const SORT_KEYS = ["Name", "Provider", "InstanceType"];

export default function BuilderTable({
  resources,
  onServerSort,
  ...boxProps
}: {
  resources: Types.BuilderListItem[];
  /** When provided, sorting is handled server side,
   * and sort updates are passed to this callback. */
  onServerSort?: (sort: {
    sort_by?: string;
    sort_desc?: boolean;
  }) => void;
} & BoxProps) {
  const [selectedResources, setSelectedResources] =
    useSelectedResources("Builder");
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
      tableKey="builders"
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
            <ResourceLink type="Builder" id={row.original.id} />
          ),
        },
        {
          id: "Provider",
          accessorKey: "info.builder_type",
          header: ({ column }) => (
            <SortableHeader column={column} title="Provider" />
          ),
        },
        {
          id: "InstanceType",
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
