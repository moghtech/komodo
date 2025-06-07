import { Page } from "@components/layouts";
import { ResourceLink } from "@components/resources/common";
import { usePermissions, useRead, useWrite } from "@lib/hooks";
import { UsableResource } from "@types";
import { DataTable, SortableHeader } from "@ui/data-table";
import { Switch } from "@ui/switch";
import { useToast } from "@ui/use-toast";
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
            size: 100,
            accessorKey: "target.type",
            header: ({ column }) => (
              <SortableHeader column={column} title="Type" />
            ),
          },
          {
            size: 200,
            accessorKey: "name",
            header: ({ column }) => (
              <SortableHeader column={column} title="Target" />
            ),
            cell: ({ row }) => (
              <ResourceLink
                type={row.original.target.type as UsableResource}
                id={row.original.target.id}
              />
            ),
          },
          {
            size: 200,
            accessorKey: "next_scheduled_run",
            header: ({ column }) => (
              <SortableHeader column={column} title="Next Run" />
            ),
            sortingFn: (a, b) => {
              const sa = a.original.next_scheduled_run;
              const sb = b.original.next_scheduled_run;

              if (!sa && !sb) return 0;
              if (!sa) return 1;
              if (!sb) return -1;

              if (sa > sb) return 1;
              else if (sa < sb) return -1;
              else return 0;
            },
            cell: ({ row }) =>
              row.original.next_scheduled_run
                ? new Date(row.original.next_scheduled_run).toLocaleString()
                : "Not Scheduled",
          },
          {
            size: 200,
            accessorKey: "schedule",
            header: ({ column }) => (
              <SortableHeader column={column} title="Schedule" />
            ),
          },
          {
            size: 100,
            accessorKey: "enabled",
            header: ({ column }) => (
              <SortableHeader column={column} title="Enabled" />
            ),
            cell: ({ row: { original: schedule } }) => (
              <ScheduleEnableSwitch
                type={schedule.target.type as UsableResource}
                id={schedule.target.id}
                enabled={schedule.enabled}
              />
            ),
          },
        ]}
      />
    </Page>
  );
}

const ScheduleEnableSwitch = ({
  type,
  id,
  enabled,
}: {
  type: UsableResource;
  id: string;
  enabled: boolean;
}) => {
  const { canWrite } = usePermissions({ type, id });
  const { toast } = useToast();
  const { mutate } = useWrite(`Update${type}`, {
    onSuccess: () => toast({ title: "Updated Schedule enabled." }),
  });
  return (
    <Switch
      checked={enabled}
      onCheckedChange={(enabled) =>
        mutate({ id, config: { schedule_enabled: enabled } })
      }
      disabled={!canWrite}
    />
  );
};
