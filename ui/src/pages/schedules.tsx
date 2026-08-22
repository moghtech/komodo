import TagsFilter from "@/components/tags/filter";
import TableTags from "@/components/tags/table";
import ListPagination from "@/components/list-pagination";
import {
  useDebouncedTermSearch,
  usePermissions,
  useRead,
  useSetTitle,
  useTagsFilter,
  useWrite,
} from "@/lib/hooks";
import { UsableResource } from "@/resources";
import ResourceLink from "@/resources/link";
import { ICONS } from "@/lib/icons";
import { DataTable, SortableHeader } from "mogh_ui";
import { Page } from "mogh_ui";
import { SearchInput } from "mogh_ui";
import { Group, Stack, Switch } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { keepPreviousData } from "@tanstack/react-query";
import { Types } from "komodo_client";
import { useEffect, useState } from "react";

const SCHEDULE_SORT_KEYS = Object.values(Types.ScheduleSortBy);

export default function Schedules() {
  useSetTitle("Schedules");

  const [page, setPage] = useState(0);

  const { search, setSearch, terms } = useDebouncedTermSearch({
    onUpdate: () => setPage(0),
  });

  const tags = useTagsFilter();

  // Server side sort, passed up from the table.
  const [sort, setSort] = useState<{
    sort_by?: Types.ScheduleSortBy;
    sort_desc?: boolean;
  }>({});

  // Set to page 0 whenever any filter or the sort changes,
  // otherwise the query can point past the last page and come back empty.
  useEffect(() => {
    setPage(0);
  }, [terms, tags, sort.sort_by, sort.sort_desc]);

  const schedules =
    useRead(
      "ListSchedules",
      {
        tags,
        terms,
        page,
        sort_by: sort.sort_by,
        sort_desc: sort.sort_desc,
      },
      {
        refetchInterval: 15_000,
        // Keep the previous rows visible while fetching after a query key
        // change (page / sort / search / filters) to prevent table flashing.
        placeholderData: keepPreviousData,
      },
    ).data ?? [];

  return (
    <Page
      icon={ICONS.Schedule}
      title="Schedules"
      description="See an overview of your scheduled Actions and Procedures."
    >
      <Stack>
        <Group justify="end">
          <ListPagination
            page={page}
            setPage={setPage}
            count={schedules.length}
          />
          <TagsFilter />
          <SearchInput value={search} onSearch={setSearch} />
        </Group>

        <DataTable
          tableKey="schedules"
          data={schedules}
          manualSorting
          onSortingStateChange={(sorting) => {
            const sort = sorting.find((s) =>
              SCHEDULE_SORT_KEYS.includes(s.id as Types.ScheduleSortBy),
            );
            setSort(
              sort
                ? {
                    sort_by: sort.id as Types.ScheduleSortBy,
                    sort_desc: sort.desc,
                  }
                : {},
            );
          }}
          columns={[
            {
              id: "Name",
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
              id: "Schedule",
              accessorKey: "schedule",
              header: ({ column }) => (
                <SortableHeader column={column} title="Schedule" />
              ),
            },
            {
              id: "NextRun",
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
              id: "Enabled",
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
