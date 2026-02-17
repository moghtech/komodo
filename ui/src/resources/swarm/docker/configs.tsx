import { useRead } from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import { ReactNode } from "react";
import { useSwarmDockerSearch } from ".";
import Section from "@/ui/section";
import { Badge, TextInput } from "@mantine/core";
import { ICONS } from "@/theme/icons";
import { DataTable, SortableHeader } from "@/ui/data-table";
import SwarmResourceLink from "@/components/swarm/link";

export default function SwarmConfigs({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const [search, setSearch] = useSwarmDockerSearch();
  const configs =
    useRead("ListSwarmConfigs", { swarm: id }, { refetchInterval: 10_000 })
      .data ?? [];

  const filtered = filterBySplit(
    configs,
    search,
    (config) => config.Name ?? config.ID ?? "Unknown",
  );

  return (
    <Section
      titleOther={titleOther}
      actions={
        <TextInput
          leftSection={<ICONS.Search size="1rem" />}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="search..."
        />
      }
    >
      <DataTable
        tableKey="swarm-configs"
        data={filtered}
        columns={[
          {
            accessorKey: "Name",
            header: ({ column }) => (
              <SortableHeader column={column} title="Name" />
            ),
            cell: ({ row }) => (
              <SwarmResourceLink
                type="Config"
                swarmId={id}
                resourceId={row.original.Name}
                name={row.original.Name}
                extra={
                  !row.original.InUse && (
                    <Badge variant="filled" color="red">
                      Unused
                    </Badge>
                  )
                }
              />
            ),
          },
          {
            accessorKey: "ID",
            header: ({ column }) => (
              <SortableHeader column={column} title="Id" />
            ),
            cell: ({ row }) => row.original.ID ?? "Unknown",
          },
          {
            accessorKey: "UpdatedAt",
            header: ({ column }) => (
              <SortableHeader column={column} title="Updated" />
            ),
          },
          {
            accessorKey: "CreatedAt",
            header: ({ column }) => (
              <SortableHeader column={column} title="Created" />
            ),
          },
        ]}
      />
    </Section>
  );
}
