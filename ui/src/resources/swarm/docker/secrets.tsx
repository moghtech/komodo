import { useRead } from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import { ReactNode } from "react";
import { useSwarmDockerSearch } from ".";
import { Section } from "mogh_ui";
import { Badge, Group } from "@mantine/core";
import { DataTable, SortableHeader } from "mogh_ui";
import SwarmResourceLink from "@/components/swarm/link";
import SearchInput from "@/ui/search-input";
import NewSwarmSecret from "@/resources/swarm/new/secret";

export default function SwarmSecrets({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const [search, setSearch] = useSwarmDockerSearch();
  const secrets =
    useRead("ListSwarmSecrets", { swarm: id }, { refetchInterval: 10_000 })
      .data ?? [];

  const filtered = filterBySplit(
    secrets,
    search,
    (secret) => secret.Name ?? secret.ID ?? "Unknown",
  );

  return (
    <Section
      titleOther={titleOther}
      actions={
        <Group>
          <NewSwarmSecret id={id} />
          <SearchInput value={search} onSearch={setSearch} />
        </Group>
      }
    >
      <DataTable
        tableKey="swarm-secrets"
        data={filtered}
        columns={[
          {
            accessorKey: "Name",
            header: ({ column }) => (
              <SortableHeader column={column} title="Name" />
            ),
            cell: ({ row }) => (
              <SwarmResourceLink
                type="Secret"
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
            size: 200,
          },
          {
            accessorKey: "ID",
            header: ({ column }) => (
              <SortableHeader column={column} title="ID" />
            ),
            cell: ({ row }) => row.original.ID ?? "Unknown",
            size: 200,
          },
          {
            accessorKey: "Driver",
            header: ({ column }) => (
              <SortableHeader column={column} title="Driver" />
            ),
            cell: ({ row }) =>
              row.original.Driver ?? (
                <div className="text-muted-foreground">None</div>
              ),
          },
          {
            accessorKey: "Templating",
            header: ({ column }) => (
              <SortableHeader column={column} title="Templating" />
            ),
            cell: ({ row }) =>
              row.original.Templating ?? (
                <div className="text-muted-foreground">None</div>
              ),
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
}
