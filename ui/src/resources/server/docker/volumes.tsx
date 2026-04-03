import { ReactNode } from "react";
import { useServerDockerSearch } from ".";
import { useRead } from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import { Section } from "mogh_ui";
import { Prune } from "../executions";
import { Badge, Group } from "@mantine/core";
import { DataTable, SortableHeader } from "mogh_ui";
import DockerResourceLink from "@/components/docker/link";
import { SearchInput } from "mogh_ui";

export default function ServerVolumes({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const [search, setSearch] = useServerDockerSearch();
  const volumes =
    useRead("ListDockerVolumes", { server: id }, { refetchInterval: 10_000 })
      .data ?? [];

  const allInUse = volumes.every((volume) => volume.in_use);

  const filtered = filterBySplit(volumes, search, (volume) => volume.name);

  return (
    <Section
      titleOther={titleOther}
      actions={
        <Group>
          {!allInUse && <Prune serverId={id} type="Volumes" />}
          <SearchInput value={search} onSearch={setSearch} />
        </Group>
      }
    >
      <DataTable
        mih="60vh"
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
                type="Volume"
                serverId={id}
                name={row.original.name}
                extra={
                  !row.original.in_use && <Badge color="red">Unused</Badge>
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
}
