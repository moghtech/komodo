import { ReactNode } from "react";
import { useServerDockerSearch } from ".";
import { useRead } from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import { fmtSizeBytes, Section } from "mogh_ui";
import { Badge, Group } from "@mantine/core";
import { Prune } from "../executions";
import { DataTable, SortableHeader } from "mogh_ui";
import DockerResourceLink from "@/components/docker/link";
import SearchInput from "@/ui/search-input";

export default function ServerImages({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const [search, setSearch] = useServerDockerSearch();
  const images =
    useRead("ListDockerImages", { server: id }, { refetchInterval: 10_000 })
      .data ?? [];

  const allInUse = images.every((image) => image.in_use);

  const filtered = filterBySplit(images, search, (image) => image.name);

  return (
    <Section
      titleOther={titleOther}
      actions={
        <Group>
          {!allInUse && <Prune serverId={id} type="Images" />}
          <SearchInput value={search} onSearch={setSearch} />
        </Group>
      }
    >
      <DataTable
        mih="60vh"
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
                type="Image"
                serverId={id}
                name={row.original.name}
                id={row.original.id}
                extra={
                  !row.original.in_use && <Badge color="red">Unused</Badge>
                }
              />
            ),
            size: 200,
          },
          {
            accessorKey: "id",
            header: ({ column }) => (
              <SortableHeader column={column} title="ID" />
            ),
          },
          {
            accessorKey: "size",
            header: ({ column }) => (
              <SortableHeader column={column} title="Size" />
            ),
            cell: ({ row }) =>
              row.original.size ? fmtSizeBytes(row.original.size) : "Unknown",
          },
        ]}
      />
    </Section>
  );
}
