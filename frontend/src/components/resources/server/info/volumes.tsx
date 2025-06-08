import { Section } from "@components/layouts";
import { DockerResourceLink } from "@components/util";
import { useRead } from "@lib/hooks";
import { Badge } from "@ui/badge";
import { DataTable, SortableHeader } from "@ui/data-table";
import { Dispatch, ReactNode, SetStateAction } from "react";
import { Prune } from "../actions";
import { Search } from "lucide-react";
import { Input } from "@ui/input";
import { filterBySplit } from "@lib/utils";

export const Volumes = ({
  id,
  titleOther,
  _search,
}: {
  id: string;
  titleOther: ReactNode;
  _search: [string, Dispatch<SetStateAction<string>>];
}) => {
  const [search, setSearch] = _search;
  const volumes =
    useRead("ListDockerVolumes", { server: id }, { refetchInterval: 10_000 })
      .data ?? [];

  const allInUse = volumes.every((volume) => volume.in_use);

  const filtered = filterBySplit(volumes, search, (volume) => volume.name);

  return (
    <Section
      titleOther={titleOther}
      actions={
        <div className="flex items-center gap-4">
          {!allInUse && <Prune server_id={id} type="Volumes" />}
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
        tableKey="server-volumes"
        data={filtered}
        columns={[
          {
            accessorKey: "name",
            header: ({ column }) => (
              <SortableHeader column={column} title="Name" />
            ),
            cell: ({ row }) => (
              <DockerResourceLink
                type="volume"
                server_id={id}
                name={row.original.name}
                extra={
                  !row.original.in_use && (
                    <Badge variant="destructive">Unused</Badge>
                  )
                }
              />
            ),
            size: 200,
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
        ]}
      />
    </Section>
  );
};
