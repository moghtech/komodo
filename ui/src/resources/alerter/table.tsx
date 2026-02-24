import { useSelectedResources } from "@/lib/hooks";
import { DataTable, SortableHeader } from "@/ui/data-table";
import { Badge, BoxProps } from "@mantine/core";
import { Types } from "komodo_client";
import ResourceLink from "@/resources/link";
import TableTags from "@/components/tags/table";
import StatusBadge from "@/ui/status-badge";

export default function AlerterTable({
  resources,
  ...boxProps
}: { resources: Types.AlerterListItem[] } & BoxProps) {
  const [_, setSelectedResources] = useSelectedResources("Alerter");
  return (
    <DataTable
      {...boxProps}
      tableKey="alerters"
      data={resources}
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
            <ResourceLink type="Alerter" id={row.original.id} />
          ),
        },
        {
          accessorKey: "info.endpoint_type",
          header: ({ column }) => (
            <SortableHeader column={column} title="Type" />
          ),
          cell: ({ row }) => (
            <Badge size="lg">{row.original.info.endpoint_type}</Badge>
          ),
        },
        {
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
