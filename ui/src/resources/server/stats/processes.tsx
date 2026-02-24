import { useRead } from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import { DataTable, SortableHeader } from "@/ui/data-table";
import SearchInput from "@/ui/search-input";
import Section from "@/ui/section";
import ShowHideButton from "@/ui/show-hide-button";
import TableSkeleton from "@/ui/table-skeleton";
import { Group } from "@mantine/core";
import { useState } from "react";

export default function ServerProcesses({ id }: { id: string }) {
  const [show, setShow] = useState(false);
  const [search, setSearch] = useState("");

  const { data: processes, isPending } = useRead(
    "ListSystemProcesses",
    {
      server: id,
    },
    { enabled: show },
  );

  const filtered = filterBySplit(processes, search, (item) => item.name);

  return (
    <Section
      title="Processes"
      titleRight={
        <Group ml={{ sm: "xl" }}>
          <SearchInput
            value={search}
            onSearch={setSearch}
            w={{ base: 200, lg: 300 }}
          />
          <ShowHideButton show={show} setShow={setShow} />
        </Group>
      }
    >
      {show && isPending && !processes && <TableSkeleton />}
      {show && !isPending && (
        <DataTable
          sortDescFirst
          tableKey="server-processes"
          data={filtered ?? []}
          columns={[
            {
              header: "Name",
              accessorKey: "name",
            },
            {
              header: "Exe",
              accessorKey: "exe",
              cell: ({ row }) => (
                <div className="overflow-hidden overflow-ellipsis">
                  {row.original.exe}
                </div>
              ),
            },
            {
              accessorKey: "cpu_perc",
              header: ({ column }) => (
                <SortableHeader column={column} title="Cpu" sortDescFirst />
              ),
              cell: ({ row }) => <>{row.original.cpu_perc.toFixed(2)}%</>,
            },
            {
              accessorKey: "mem_mb",
              header: ({ column }) => (
                <SortableHeader column={column} title="Memory" sortDescFirst />
              ),
              cell: ({ row }) => (
                <>
                  {row.original.mem_mb > 1000
                    ? `${(row.original.mem_mb / 1024).toFixed(2)} GB`
                    : `${row.original.mem_mb.toFixed(2)} MB`}
                </>
              ),
            },
          ]}
        />
      )}
    </Section>
  );
}
