import { useSelectedResources } from "@/lib/hooks";
import { DataTable, SortableHeader } from "mogh_ui";
import { Types } from "komodo_client";
import { fmtVersion } from "@/lib/formatting";
import { BuildComponents } from ".";
import TableTags from "@/components/tags/table";
import { BoxProps } from "@mantine/core";
import ResourceLink from "@/resources/link";
import FileSource from "@/components/file-source";
import { RowSelectionState } from "@tanstack/react-table";
import { setSelectedStateHandler } from "@/lib/utils";

const SORT_KEYS = ["Name", "Source", "State"];

export default function BuildTable({
  resources,
  onServerSort,
  ...boxProps
}: {
  resources: Types.BuildListItem[];
  /** When provided, sorting is handled server side,
   * and sort updates are passed to this callback. */
  onServerSort?: (sort: {
    sort_by?: string;
    sort_desc?: boolean;
  }) => void;
} & BoxProps) {
  const [selectedResources, setSelectedResources] =
    useSelectedResources("Build");
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
      tableKey="builds"
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
          header: ({ column }) => (
            <SortableHeader column={column} title="Name" />
          ),
          id: "Name",
          accessorKey: "name",
          cell: ({ row }) => <ResourceLink type="Build" id={row.original.id} />,
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
          header: "Version",
          accessorFn: ({ info }) => fmtVersion(info.version),
          size: 120,
        },
        {
          id: "State",
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
