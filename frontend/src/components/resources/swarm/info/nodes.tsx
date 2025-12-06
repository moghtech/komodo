import { Section } from "@components/layouts";
import { useRead } from "@lib/hooks";
import { DataTable, SortableHeader } from "@ui/data-table";
import { Dispatch, ReactNode, SetStateAction } from "react";
import { Search } from "lucide-react";
import { Input } from "@ui/input";
import { filterBySplit } from "@lib/utils";
import { SwarmResourceLink } from "..";

export const SwarmNodes = ({
  id,
  titleOther,
  _search,
}: {
  id: string;
  titleOther: ReactNode;
  _search: [string, Dispatch<SetStateAction<string>>];
}) => {
  const [search, setSearch] = _search;
  const nodes =
    useRead("ListSwarmNodes", { swarm: id }, { refetchInterval: 10_000 })
      .data ?? [];

  const filtered = filterBySplit(
    nodes,
    search,
    (node) => node.Name ?? node.Hostname ?? node.ID ?? "Unknown"
  );

  return (
    <Section
      titleOther={titleOther}
      actions={
        <div className="flex items-center gap-4 flex-wrap">
          <div className="relative">
            <Search className="w-4 absolute top-[50%] left-3 -translate-y-[50%] text-muted-foreground" />
            <Input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="search..."
              className="pl-8 w-[200px] lg:w-[300px]"
            />
          </div>
        </div>
      }
    >
      <DataTable
        containerClassName="min-h-[60vh]"
        tableKey="swarm-nodes"
        data={filtered}
        columns={[
          {
            accessorKey: "Hostname",
            header: ({ column }) => (
              <SortableHeader column={column} title="Hostname" />
            ),
            cell: ({ row }) => (
              <SwarmResourceLink
                type="Node"
                swarm_id={id}
                resource_id={row.original.ID}
                name={row.original.Hostname}
              />
            ),
            size: 200,
          },
          {
            accessorKey: "ID",
            header: ({ column }) => (
              <SortableHeader column={column} title="Id" />
            ),
            cell: ({ row }) => row.original.ID ?? "Unknown",
            size: 200,
          },
          {
            accessorKey: "Role",
            header: ({ column }) => (
              <SortableHeader column={column} title="Role" />
            ),
            cell: ({ row }) => row.original.Role ?? "Unknown",
          },
          {
            accessorKey: "Availability",
            header: ({ column }) => (
              <SortableHeader column={column} title="Availability" />
            ),
            cell: ({ row }) => row.original.Availability ?? "Unknown",
          },
          {
            accessorKey: "State",
            header: ({ column }) => (
              <SortableHeader column={column} title="State" />
            ),
            cell: ({ row }) => row.original.State ?? "Unknown",
          },
          {
            accessorKey: "UpdatedAt",
            header: ({ column }) => (
              <SortableHeader column={column} title="Updated" />
            ),
            cell: ({ row }) =>
              row.original.UpdatedAt
                ? new Date(row.original.UpdatedAt).toLocaleString()
                : "Unknown",
            size: 200,
          },
          {
            accessorKey: "CreatedAt",
            header: ({ column }) => (
              <SortableHeader column={column} title="Created" />
            ),
            cell: ({ row }) =>
              row.original.CreatedAt
                ? new Date(row.original.CreatedAt).toLocaleString()
                : "Unknown",
            size: 200,
          },
        ]}
      />
    </Section>
  );
};
