import { useRead } from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import { ReactNode } from "react";
import { useSwarmDockerSearch } from ".";
import Section from "@/ui/section";
import { TextInput } from "@mantine/core";
import { ICONS } from "@/theme/icons";
import { DataTable, SortableHeader } from "@/ui/data-table";
import SwarmResourceLink from "@/components/swarm/link";
import StatusBadge from "@/ui/status-badge";
import {
  swarmNodeAvailabilityIntention,
  swarmNodeRoleIntention,
  swarmNodeStateIntention,
} from "@/lib/color";

export default function SwarmNodes({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const [search, setSearch] = useSwarmDockerSearch();
  const nodes =
    useRead("ListSwarmNodes", { swarm: id }, { refetchInterval: 10_000 })
      .data ?? [];

  const filtered = filterBySplit(
    nodes,
    search,
    (node) => node.Name ?? node.Hostname ?? node.ID ?? "Unknown",
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
        tableKey="swarm-nodes"
        data={filtered}
        columns={[
          {
            accessorKey: "Hostname",
            header: ({ column }) => (
              <SortableHeader column={column} title="Hostname" />
            ),
            cell: ({ row }) => (
              <SwarmResourceLink
                type="Node"
                swarmId={id}
                resourceId={row.original.ID}
                name={row.original.Hostname}
              />
            ),
            size: 200,
          },
          {
            accessorKey: "ID",
            header: ({ column }) => (
              <SortableHeader column={column} title="Id" />
            ),
            cell: ({ row }) => row.original.ID ?? "Unknown",
            size: 200,
          },
          {
            accessorKey: "Role",
            header: ({ column }) => (
              <SortableHeader column={column} title="Role" />
            ),
            cell: ({ row }) => (
              <StatusBadge
                text={row.original.Role}
                intent={swarmNodeRoleIntention(row.original.Role)}
              />
            ),
          },
          {
            accessorKey: "State",
            header: ({ column }) => (
              <SortableHeader column={column} title="State" />
            ),
            cell: ({ row }) => (
              <StatusBadge
                text={row.original.State}
                intent={swarmNodeStateIntention(row.original.State)}
              />
            ),
          },
          {
            accessorKey: "Availability",
            header: ({ column }) => (
              <SortableHeader column={column} title="Availability" />
            ),
            cell: ({ row }) => (
              <StatusBadge
                text={row.original.Availability}
                intent={swarmNodeAvailabilityIntention(
                  row.original.Availability,
                )}
              />
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
          // {
          //   accessorKey: "CreatedAt",
          //   header: ({ column }) => (
          //     <SortableHeader column={column} title="Created" />
          //   ),
          //   cell: ({ row }) =>
          //     row.original.CreatedAt
          //       ? new Date(row.original.CreatedAt).toLocaleString()
          //       : "Unknown",
          //   size: 200,
          // },
        ]}
      />
    </Section>
  );
}
