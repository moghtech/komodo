import { Page } from "@components/layouts";
import { ResourceLink } from "@components/resources/common";
import { TableTags, TagsFilter } from "@components/tags";
import {
  usePermissions,
  useRead,
  useSetTitle,
  useTags,
  useWrite,
} from "@lib/hooks";
import { filterBySplit } from "@lib/utils";
import { UsableResource } from "@types";
import { DataTable, SortableHeader } from "@ui/data-table";
import { Input } from "@ui/input";
import { Switch } from "@ui/switch";
import { useToast } from "@ui/use-toast";
import { CalendarDays, Search } from "lucide-react";
import { useState } from "react";

export default function SchedulesPage() {
  useSetTitle("Schedules");
  const [search, set] = useState("");
  const { tags } = useTags();
  const schedules = useRead("ListSchedules", { tags }).data;
  const filtered = filterBySplit(schedules ?? [], search, (item) => item.name);
  return (
    <Page
      icon={<CalendarDays className="w-8" />}
      title="Schedules"
      subtitle={
        <div className="text-muted-foreground">
          See an overview of your scheduled tasks.
        </div>
      }
    >
      <div className="flex flex-col gap-4">
        <div className="flex flex-wrap gap-4 items-center justify-end">
          <div className="flex items-center gap-4 flex-wrap">
            <TagsFilter />
            <div className="relative">
              <Search className="w-4 absolute top-[50%] left-3 -translate-y-[50%] text-muted-foreground" />
              <Input
                value={search}
                onChange={(e) => set(e.target.value)}
                placeholder="search..."
                className="pl-8 w-[200px] lg:w-[300px]"
              />
            </div>
          </div>
        </div>
        <DataTable
          tableKey="schedules"
          data={filtered}
          columns={[
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
              accessorKey: "schedule",
              header: ({ column }) => (
                <SortableHeader column={column} title="Schedule" />
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
            {
              header: "Tags",
              cell: ({ row }) => <TableTags tag_ids={row.original.tags} />,
            },
          ]}
        />
      </div>
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
