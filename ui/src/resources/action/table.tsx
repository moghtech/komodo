import { Types } from "komodo_client";
import { DataTable, SortableHeader } from "@/ui/data-table";
import { ResourceLink } from "@/resources/common";
import { useSelectedResources, useTags } from "@/lib/hooks";
import { ActionComponents } from ".";
import Tags from "@/components/tags";

export const ActionTable = ({
  actions,
}: {
  actions: Types.ActionListItem[];
}) => {
  const [_, setSelectedResources] = useSelectedResources("Action");
  const { toggle_tag } = useTags();
  return (
    <DataTable
      tableKey="actions"
      data={actions}
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
            <ResourceLink type="Action" id={row.original.id} />
          ),
        },
        {
          accessorKey: "info.state",
          header: ({ column }) => (
            <SortableHeader column={column} title="State" />
          ),
          cell: ({ row }) => <ActionComponents.State id={row.original.id} />,
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
          cell: ({ row }) => (
            <Tags tag_ids={row.original.tags} onBadgeClick={toggle_tag} />
          ),
        },
      ]}
    />
  );
};
