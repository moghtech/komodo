import { ReactNode } from "react";
import { useServerDockerSearch } from ".";
import { useRead } from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import Section from "@/ui/section";
import { Badge, Group, TextInput } from "@mantine/core";
import { Prune } from "../executions";
import { ICONS } from "@/theme/icons";
import { DataTable, SortableHeader } from "@/ui/data-table";
import DockerResourceLink from "@/components/docker/link";
import { fmtSizeBytes } from "@/lib/formatting";

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
          <TextInput
            leftSection={<ICONS.Search size="1rem" />}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="search..."
          />
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
              <SortableHeader column={column} title="Id" />
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
