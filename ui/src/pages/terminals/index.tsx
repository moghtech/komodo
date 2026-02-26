import TerminalTargetLink from "@/pages/terminals/target-link";
import { fmtDateWithMinutes } from "@/lib/formatting";
import { useRead, useSetTitle } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { filterBySplit, terminalLink } from "@/lib/utils";
import { DataTable, SortableHeader } from "@/ui/data-table";
import Page from "@/ui/page";
import { Group, Stack, Text } from "@mantine/core";
import { useState } from "react";
import { Link } from "react-router-dom";
import DeleteTerminal from "./delete";
import BatchDeleteAllTerminals from "./batch-delete";
import CreateTerminal from "./create";
import SearchInput from "@/ui/search-input";

export default function Terminals() {
  useSetTitle("Terminals");
  const [search, setSearch] = useState("");
  const { data: terminals, refetch, isPending } = useRead("ListTerminals", {});
  const filtered = filterBySplit(terminals ?? [], search, (item) => item.name);
  return (
    <Page
      title="Terminals"
      icon={ICONS.Terminal}
      description="Manage Terminals across all your Servers."
    >
      <Stack>
        <Group justify="space-between">
          <Group>
            <CreateTerminal />
            <BatchDeleteAllTerminals
              refetch={refetch}
              noTerminals={!terminals?.length}
            />
          </Group>
          <SearchInput value={search} onSearch={setSearch} />
        </Group>

        <DataTable
          tableKey="terminals"
          data={filtered}
          loading={isPending}
          columns={[
            {
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
              size: 200,
              accessorKey: "target",
              header: ({ column }) => (
                <SortableHeader column={column} title="Target" />
              ),
              cell: ({ row }) => (
                <TerminalTargetLink target={row.original.target} />
              ),
            },
            {
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
              size: 100,
              accessorKey: "size",
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
