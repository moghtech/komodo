import { ReactNode } from "react";
import { useServerDockerSearch } from ".";
import { useDockerSelectionState, useRead } from "@/lib/hooks";
import DockerBatchExecutions from "@/components/docker/batch-executions";
import { filterBySplit } from "mogh_ui";
import { fmtSizeBytes, Section } from "mogh_ui";
import { Badge, Group } from "@mantine/core";
import { Prune } from "../executions";
import { DataTable, SortableHeader } from "mogh_ui";
import DockerResourceLink from "@/components/docker/link";
import { SearchInput } from "mogh_ui";

export default function ServerImages({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const [search, setSearch] = useServerDockerSearch();
  const selectionState = useDockerSelectionState("Image");
  const images =
    useRead("ListImages", { server: id }, { refetchInterval: 10_000 }).data ??
    [];

  const allInUse = images.every((image) => image.in_use);

  const filtered = filterBySplit(images, search, (image) => image.name);

  return (
    <Section titleOther={titleOther}>
      <Group justify="space-between">
        <Group>
          <DockerBatchExecutions type="Image" />
          {!allInUse && <Prune serverId={id} type="Images" />}
        </Group>

        <SearchInput value={search} onSearch={setSearch} />
      </Group>

      <DataTable
        mih="60vh"
        tableKey="server-images"
        data={filtered}
        selectOptions={{
          selectKey: ({ name }) => `${id} ${name}`,
          state: selectionState,
        }}
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
