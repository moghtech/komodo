import { DataTable, SortableHeader } from "@ui/data-table";
import { TableTags } from "@components/tags";
import { ResourceLink } from "../common";
import { ProcedureComponents } from ".";
import { Types } from "komodo_client";
import { useSelectedResources } from "@lib/hooks";

export const ProcedureTable = ({
  procedures,
}: {
  procedures: Types.ProcedureListItem[];
}) => {
  const [_, setSelectedResources] = useSelectedResources("Procedure");

  return (
    <DataTable
      tableKey="procedures"
      data={procedures}
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
          cell: ({ row }) => (
            <ResourceLink type="Procedure" id={row.original.id} />
          ),
        },
        {
          accessorKey: "info.state",
          header: ({ column }) => (
            <SortableHeader column={column} title="State" />
          ),
          cell: ({ row }) => <ProcedureComponents.State id={row.original.id} />,
        },
        {
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
          cell: ({ row }) => <TableTags tag_ids={row.original.tags} />,
        },
      ]}
    />
  );
};
