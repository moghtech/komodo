import TagsFilter from "@/components/tags/filter";
import TableTags from "@/components/tags/table";
import {
  usePermissions,
  useRead,
  useSetTitle,
  useTags,
  useWrite,
} from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import { UsableResource } from "@/resources";
import ResourceLink from "@/resources/link";
import { ICONS } from "@/theme/icons";
import { DataTable, SortableHeader } from "@/ui/data-table";
import Page from "@/ui/page";
import SearchInput from "@/ui/search-input";
import { Group, Stack, Switch } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useState } from "react";

export default function Schedules() {
  useSetTitle("Schedules");
  const [search, setSearch] = useState("");
  const { tags } = useTags();
  const schedules = useRead("ListSchedules", { tags }).data;
  const filtered = filterBySplit(schedules ?? [], search, (item) => item.name);

  return (
    <Page
      icon={ICONS.Schedule}
      title="Schedules"
      description="See an overview of your scheduled Actions and Procedures."
    >
      <Stack>
        <Group justify="end">
          <TagsFilter />
          <SearchInput value={search} onSearch={setSearch} />
        </Group>

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
              cell: ({ row }) => <TableTags tagIds={row.original.tags} />,
            },
          ]}
        />
      </Stack>
    </Page>
  );
}

function ScheduleEnableSwitch({
  type,
  id,
  enabled,
}: {
  type: UsableResource;
  id: string;
  enabled: boolean;
}) {
  const { canWrite } = usePermissions({ type, id });
  const { mutate } = useWrite(`Update${type}`, {
    onSuccess: () =>
      notifications.show({ message: "Updated Schedule enabled." }),
  });
  return (
    <Switch
      checked={enabled}
      onChange={(e) =>
        mutate({ id, config: { schedule_enabled: e.target.checked } })
      }
      disabled={!canWrite}
    />
  );
}
