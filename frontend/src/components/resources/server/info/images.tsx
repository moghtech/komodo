import { Section } from "@components/layouts";
import { DockerResourceLink } from "@components/util";
import { format_size_bytes } from "@lib/formatting";
import { useRead } from "@lib/hooks";
import { Badge } from "@ui/badge";
import { DataTable, SortableHeader } from "@ui/data-table";
import { Dispatch, ReactNode, SetStateAction } from "react";
import { Prune } from "../actions";
import { Search } from "lucide-react";
import { Input } from "@ui/input";
import { filterBySplit } from "@lib/utils";

export const Images = ({
  id,
  titleOther,
  _search,
}: {
  id: string;
  titleOther: ReactNode;
  _search: [string, Dispatch<SetStateAction<string>>];
}) => {
  const [search, setSearch] = _search;
  const images =
    useRead("ListDockerImages", { server: id }, { refetchInterval: 10_000 })
      .data ?? [];

  const allInUse = images.every((image) => image.in_use);

  const filtered = filterBySplit(images, search, (image) => image.name);

  return (
    <Section
      titleOther={titleOther}
      actions={
        <div className="flex items-center gap-4">
          {!allInUse && <Prune server_id={id} type="Images" />}
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
        tableKey="server-images"
        data={filtered}
        columns={[
          {
            accessorKey: "name",
            header: ({ column }) => (
              <SortableHeader column={column} title="Name" />
            ),
            cell: ({ row }) => (
              <DockerResourceLink
                type="image"
                server_id={id}
                name={row.original.name}
                id={row.original.id}
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
            accessorKey: "id",
            header: ({ column }) => (
              <SortableHeader column={column} title="Id" />
            ),
          },
          {
            accessorKey: "size",
            header: ({ column }) => (
              <SortableHeader column={column} title="Size" />
            ),
            cell: ({ row }) =>
              row.original.size
                ? format_size_bytes(row.original.size)
                : "Unknown",
          },
        ]}
      />
    </Section>
  );
};
