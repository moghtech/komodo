import { DataTable, SortableHeader } from "@ui/data-table";
import { ResourceLink } from "../common";
import { TableTags } from "@components/tags";
import { Types } from "komodo_client";
import { useSelectedResources } from "@lib/hooks";

export const AlerterTable = ({
  alerters,
}: {
  alerters: Types.AlerterListItem[];
}) => {
  const [selectedResources, setSelectedResources] = useSelectedResources("Alerter");
  return (
    <DataTable
      tableKey="alerters"
      data={alerters}
      selectOptions={{
        selectKey: ({ name }) => name,
        selected: selectedResources,
        onSelect: setSelectedResources,
      }}
      columns={[
        {
          accessorKey: "name",
          header: ({ column }) => (
            <SortableHeader column={column} title="Name" />
          ),
          cell: ({ row }) => (
            <ResourceLink type="Alerter" id={row.original.id} />
          ),
        },
        {
          accessorKey: "info.endpoint_type",
          header: ({ column }) => (
            <SortableHeader column={column} title="Type" />
          ),
        },
        {
          accessorKey: "info.enabled",
          header: ({ column }) => (
            <SortableHeader column={column} title="Enabled" />
          ),
        },
        {
          header: "Tags",
          cell: ({ row }) => <TableTags tag_ids={row.original.tags} />,
        },
      ]}
    />
  );
};
