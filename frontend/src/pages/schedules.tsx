import { Page } from "@components/layouts";
import { ResourceLink } from "@components/resources/common";
import { useRead } from "@lib/hooks";
import { UsableResource } from "@types";
import { DataTable, SortableHeader } from "@ui/data-table";
import { CalendarDays } from "lucide-react";

export default function SchedulesPage() {
  const schedules = useRead("ListSchedules", { tags: [] }).data;
  return (
    <Page title="Schedules" icon={<CalendarDays className="w-8" />}>
      <DataTable
        tableKey="schedules"
        data={schedules ?? []}
        columns={[
          {
            accessorKey: "target.type",
            header: ({ column }) => (
              <SortableHeader column={column} title="Type" />
            ),
          },
          {
            accessorKey: "target.id",
            header: "Target",
            cell: ({ row }) => (
              <ResourceLink
                type={row.original.target.type as UsableResource}
                id={row.original.target.id}
              />
            ),
          },
          {
            accessorKey: "next_scheduled_run",
            header: ({ column }) => (
              <SortableHeader column={column} title="Next Run" />
            ),
            cell: ({ row }) =>
              row.original.next_scheduled_run
                ? new Date(row.original.next_scheduled_run).toLocaleString()
                : "Not Scheduled",
          },
          {
            accessorKey: "enabled",
            header: ({ column }) => (
              <SortableHeader column={column} title="Enabled" />
            ),
          },
        ]}
      />
    </Page>
  );
}
