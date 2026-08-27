import TerminalTargetLink from "@/pages/terminals/target-link";
import ListPagination from "@/components/list-pagination";
import { useDebouncedTermSearch, useRead, useSetTitle } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { terminalLink } from "@/lib/utils";
import { DataTable, fmtDateWithMinutes, SortableHeader } from "mogh_ui";
import { Page } from "mogh_ui";
import { Group, Stack, Text } from "@mantine/core";
import { keepPreviousData } from "@tanstack/react-query";
import { Types } from "komodo_client";
import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import DeleteTerminal from "./delete";
import BatchDeleteAllTerminals from "./batch-delete";
import NewTerminal from "./new";
import { SearchInput } from "mogh_ui";

const TERMINAL_SORT_KEYS = Object.values(Types.TerminalSortBy);

export default function Terminals() {
  useSetTitle("Terminals");

  const [page, setPage] = useState(0);

  const { search, setSearch, terms } = useDebouncedTermSearch({
    onUpdate: () => setPage(0),
  });

  // Server side sort, passed up from the table.
  const [sort, setSort] = useState<{
    sort_by?: Types.TerminalSortBy;
    sort_desc?: boolean;
  }>({});

  // Set to page 0 whenever the search or the sort changes,
  // otherwise the query can point past the last page and come back empty.
  useEffect(() => {
    setPage(0);
  }, [terms, sort.sort_by, sort.sort_desc]);

  const {
    data: terminals,
    refetch,
    isPending,
  } = useRead(
    "ListTerminals",
    {
      terms,
      page,
      sort_by: sort.sort_by,
      sort_desc: sort.sort_desc,
    },
    {
      refetchInterval: 15_000,
      // Keep the previous rows visible while fetching after a query key
      // change (page / sort / search) to prevent table flashing.
      placeholderData: keepPreviousData,
    },
  );

  return (
    <Page
      title="Terminals"
      icon={ICONS.Terminal}
      description="Manage terminals across all servers."
    >
      <Stack>
        <Group justify="space-between">
          <Group>
            <NewTerminal />
            <BatchDeleteAllTerminals
              refetch={refetch}
              noTerminals={!terminals?.length}
            />
          </Group>
          <Group>
            <ListPagination
              page={page}
              setPage={setPage}
              count={terminals?.length ?? 0}
            />
            <SearchInput value={search} onSearch={setSearch} />
          </Group>
        </Group>

        <DataTable
          tableKey="terminals"
          data={terminals ?? []}
          loading={isPending}
          manualSorting
          onSortingStateChange={(sorting) => {
            const sort = sorting.find((s) =>
              TERMINAL_SORT_KEYS.includes(s.id as Types.TerminalSortBy),
            );
            setSort(
              sort
                ? {
                    sort_by: sort.id as Types.TerminalSortBy,
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
                <SortableHeader column={column} title="Name" />
              ),
              cell: ({ row }) => (
                <Link
                  to={terminalLink(row.original)}
                  onClick={(e) => {
                    e.stopPropagation();
                  }}
                >
                  <Group className="hover-underline" fz="md" wrap="nowrap">
                    <ICONS.Terminal size="1rem" />
                    {row.original.name}
                  </Group>
                </Link>
              ),
            },
            {
              id: "Target",
              accessorKey: "target",
              header: ({ column }) => (
                <SortableHeader column={column} title="Target" />
              ),
              cell: ({ row }) => (
                <TerminalTargetLink target={row.original.target} />
              ),
            },
            {
              id: "Command",
              accessorKey: "command",
              header: ({ column }) => (
                <SortableHeader column={column} title="Command" />
              ),
              cell: ({ row }) => (
                <Text ff="monospace" fz="sm">
                  {row.original.command}
                </Text>
              ),
            },
            {
              id: "Size",
              accessorKey: "stored_size_kb",
              header: ({ column }) => (
                <SortableHeader column={column} title="Size" />
              ),
              cell: ({
                row: {
                  original: { stored_size_kb },
                },
              }) => (
                <span className="font-mono px-2 py-1 bg-secondary rounded-md">
                  {stored_size_kb.toFixed()} KiB
                </span>
              ),
            },
            {
              id: "Created",
              accessorKey: "created_at",
              header: ({ column }) => (
                <SortableHeader column={column} title="Created" />
              ),
              cell: ({
                row: {
                  original: { created_at },
                },
              }) => fmtDateWithMinutes(new Date(created_at)),
            },
            {
              header: "Delete",
              cell: ({ row }) => (
                <DeleteTerminal
                  target={row.original.target}
                  terminal={row.original.name}
                  refetch={refetch}
                />
              ),
            },
          ]}
        />
      </Stack>
    </Page>
  );
}
