import { Section } from "@components/layouts";
import { DockerResourceLink } from "@components/util";
import { useRead } from "@lib/hooks";
import { Badge } from "@ui/badge";
import { DataTable, SortableHeader } from "@ui/data-table";
import { Dispatch, ReactNode, SetStateAction } from "react";
import { Prune } from "../actions";
import { filterBySplit } from "@lib/utils";
import { Search } from "lucide-react";
import { Input } from "@ui/input";

export const Networks = ({
  id,
  titleOther,
  _search
}: {
  id: string;
  titleOther: ReactNode;
  _search: [string, Dispatch<SetStateAction<string>>];
}) => {
  const [search, setSearch] = _search;
  const networks =
    useRead("ListDockerNetworks", { server: id }, { refetchInterval: 10_000 })
      .data ?? [];

  const allInUse = networks.every((network) =>
    // this ignores networks that come in with no name, but they should all come in with name
    !network.name
      ? true
      : ["none", "host", "bridge"].includes(network.name)
        ? true
        : network.in_use
  );

  const filtered = filterBySplit(
    networks,
    search,
    (network) => network.name ?? ""
  );

  return (
    <Section
      titleOther={titleOther}
      actions={
        <div className="flex items-center gap-4">
          {!allInUse && <Prune server_id={id} type="Networks" />}
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
        tableKey="server-networks"
        data={filtered}
        columns={[
          {
            accessorKey: "name",
            header: ({ column }) => (
              <SortableHeader column={column} title="Name" />
            ),
            cell: ({ row }) => (
              <div className="flex items-center gap-2">
                <DockerResourceLink
                  type="network"
                  server_id={id}
                  name={row.original.name}
                  extra={
                    ["none", "host", "bridge"].includes(
                      row.original.name ?? ""
                    ) ? (
                      <Badge variant="outline">System</Badge>
                    ) : (
                      !row.original.in_use && (
                        <Badge variant="destructive">Unused</Badge>
                      )
                    )
                  }
                />
              </div>
            ),
            size: 300,
          },
          {
            accessorKey: "driver",
            header: ({ column }) => (
              <SortableHeader column={column} title="Driver" />
            ),
          },
          {
            accessorKey: "scope",
            header: ({ column }) => (
              <SortableHeader column={column} title="Scope" />
            ),
          },
          {
            accessorKey: "attachable",
            header: ({ column }) => (
              <SortableHeader column={column} title="Attachable" />
            ),
          },
          {
            accessorKey: "ipam_driver",
            header: ({ column }) => (
              <SortableHeader column={column} title="IPAM Driver" />
            ),
          },
        ]}
      />
    </Section>
  );
};
