import { useRead, useSelectedResources } from "@lib/hooks";
import { DataTable, SortableHeader } from "@ui/data-table";
import { ResourceLink, StandardSource } from "../common";
import { TableTags } from "@components/tags";
import { StackComponents, UpdateAvailable } from ".";
import { Types } from "komodo_client";
import { useCallback } from "react";

/** Returns a numeric priority for stack states to enable logical sorting.
 * Lower numbers = higher priority (shown first when sorting ascending).
 * Priority groups:
 * - Active/Running states (0-19)
 * - Inactive but configured states (20-39)
 * - Down/Unknown states (40+)
 */
const getStackStatePriority = (state: Types.StackState): number => {
  switch (state) {
    // Active states - highest priority
    case Types.StackState.Running:
      return 0;
    case Types.StackState.Deploying:
      return 1;
    case Types.StackState.Restarting:
      return 2;
    case Types.StackState.Unhealthy:
      return 3;

    // Inactive but configured states - medium priority
    case Types.StackState.Paused:
      return 20;
    case Types.StackState.Stopped:
      return 21;
    case Types.StackState.Created:
      return 22;

    // Down/problematic states - lowest priority
    case Types.StackState.Down:
      return 40;
    case Types.StackState.Dead:
      return 41;
    case Types.StackState.Removing:
      return 42;
    case Types.StackState.Unknown:
      return 43;

    default:
      return 99;
  }
};

export const StackTable = ({ stacks }: { stacks: Types.StackListItem[] }) => {
  const servers = useRead("ListServers", {}).data;
  const serverName = useCallback(
    (id: string) => servers?.find((server) => server.id === id)?.name,
    [servers]
  );

  const [_, setSelectedResources] = useSelectedResources("Stack");

  return (
    <DataTable
      tableKey="Stacks"
      data={stacks}
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
          cell: ({ row }) => {
            return (
              <div className="flex items-center justify-between gap-2">
                <ResourceLink type="Stack" id={row.original.id} />
                <UpdateAvailable id={row.original.id} small />
              </div>
            );
          },
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
          header: ({ column }) => (
            <SortableHeader column={column} title="Server" />
          ),
          accessorKey: "info.server_id",
          sortingFn: (a, b) => {
            const sa = serverName(a.original.info.server_id);
            const sb = serverName(b.original.info.server_id);

            if (!sa && !sb) return 0;
            if (!sa) return 1;
            if (!sb) return -1;

            if (sa > sb) return 1;
            else if (sa < sb) return -1;
            else return 0;
          },
          cell: ({ row }) => (
            <ResourceLink type="Server" id={row.original.info.server_id} />
          ),
          size: 200,
        },
        {
          accessorKey: "info.state",
          header: ({ column }) => (
            <SortableHeader column={column} title="State" />
          ),
          sortingFn: (a, b) => {
            const priorityA = getStackStatePriority(a.original.info.state);
            const priorityB = getStackStatePriority(b.original.info.state);
            return priorityA - priorityB;
          },
          cell: ({ row }) => <StackComponents.State id={row.original.id} />,
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
