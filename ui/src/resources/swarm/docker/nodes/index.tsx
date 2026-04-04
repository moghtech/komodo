import { useRead } from "@/lib/hooks";
import { filterBySplit } from "mogh_ui";
import { ReactNode, useState } from "react";
import { useSwarmDockerSearch } from "..";
import { Section } from "mogh_ui";
import { DataTable, SortableHeader } from "mogh_ui";
import SwarmResourceLink from "@/components/swarm/link";
import { StatusBadge } from "mogh_ui";
import {
  swarmNodeAvailabilityIntention,
  swarmNodeRoleIntention,
  swarmNodeStateIntention,
} from "@/lib/color";
import { SearchInput } from "mogh_ui";
import { RowSelectionState } from "@tanstack/react-table";
import UpdateSwarmNode from "./update";
import { HoverCard } from "@mantine/core";
import { LabelsGroup } from "mogh_ui";

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

  const selectState = useState<RowSelectionState>({});

  const filtered = filterBySplit(
    nodes,
    search,
    (node) => node.Name ?? node.Hostname ?? node.ID ?? "Unknown",
  );

  return (
    <Section
      titleOther={titleOther}
      titleRight={
        <UpdateSwarmNode swarm={id} nodes={Object.keys(selectState[0])} />
      }
      actions={<SearchInput value={search} onSearch={setSearch} />}
    >
      <DataTable
        tableKey="swarm-nodes"
        data={filtered}
        selectOptions={{
          selectKey: (node) =>
            node.Name ?? node.Hostname ?? node.ID ?? "Unknown",
          state: selectState,
        }}
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
              <SortableHeader column={column} title="ID" />
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
            accessorKey: "Labels",
            header: ({ column }) => (
              <SortableHeader column={column} title="Labels" />
            ),
            cell: ({ row }) => {
              const labels = Object.entries(row.original.Labels ?? {}).sort();
              return (
                <HoverCard position="bottom-start" disabled={labels.length < 3}>
                  <HoverCard.Target>
                    <LabelsGroup
                      labels={labels.slice(0, 2)}
                      showEllipsis={labels.length > 2}
                      wrap="nowrap"
                    />
                  </HoverCard.Target>
                  <HoverCard.Dropdown
                    maw={{ base: "calc(100vw - 100px)", xs: 400 }}
                  >
                    <LabelsGroup labels={labels.slice(2)} />
                  </HoverCard.Dropdown>
                </HoverCard>
              );
            },
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
