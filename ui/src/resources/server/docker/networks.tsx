import { ReactNode } from "react";
import { useServerDockerSearch } from ".";
import { useRead } from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import Section from "@/ui/section";
import { Prune } from "../executions";
import { Badge, Group } from "@mantine/core";
import { DataTable, SortableHeader } from "@/ui/data-table";
import DockerResourceLink from "@/components/docker/link";
import SearchInput from "@/ui/search-input";

export default function ServerNetworks({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const [search, setSearch] = useServerDockerSearch();
  const networks =
    useRead("ListDockerNetworks", { server: id }, { refetchInterval: 10_000 })
      .data ?? [];

  const allInUse = networks.every((network) =>
    // this ignores networks that come in with no name, but they should all come in with name
    !network.name
      ? true
      : ["none", "host", "bridge"].includes(network.name)
        ? true
        : network.in_use,
  );

  const filtered = filterBySplit(
    networks,
    search,
    (network) => network.name ?? "",
  );

  return (
    <Section
      titleOther={titleOther}
      actions={
        <Group>
          {!allInUse && <Prune serverId={id} type="Networks" />}
          <SearchInput value={search} onSearch={setSearch} />
        </Group>
      }
    >
      <DataTable
        mih="60vh"
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
                  type="Network"
                  serverId={id}
                  name={row.original.name}
                  extra={
                    ["none", "host", "bridge"].includes(
                      row.original.name ?? "",
                    ) ? (
                      <Badge>System</Badge>
                    ) : (
                      !row.original.in_use && <Badge color="red">Unused</Badge>
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
}
