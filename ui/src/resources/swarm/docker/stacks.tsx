import { useRead } from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import { ReactNode } from "react";
import { useSwarmDockerSearch } from ".";
import { Section } from "mogh_ui";
import { DataTable, SortableHeader } from "mogh_ui";
import SwarmResourceLink from "@/components/swarm/link";
import { StatusBadge } from "mogh_ui";
import { swarmStateIntention } from "@/lib/color";
import SearchInput from "@/ui/search-input";

export default function SwarmStacks({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const [search, setSearch] = useSwarmDockerSearch();
  const stacks =
    useRead("ListSwarmStacks", { swarm: id }, { refetchInterval: 10_000 })
      .data ?? [];

  const filtered = filterBySplit(
    stacks,
    search,
    (stack) => stack.Name ?? "Unknown",
  );

  return (
    <Section
      titleOther={titleOther}
      actions={<SearchInput value={search} onSearch={setSearch} />}
    >
      <DataTable
        tableKey="swarm-stacks"
        data={filtered}
        columns={[
          {
            accessorKey: "Name",
            header: ({ column }) => (
              <SortableHeader column={column} title="Name" />
            ),
            cell: ({ row }) => (
              <SwarmResourceLink
                type="Stack"
                swarmId={id}
                resourceId={row.original.Name}
                name={row.original.Name}
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
                intent={swarmStateIntention(row.original.State)}
              />
            ),
          },
          {
            accessorKey: "Services",
            header: ({ column }) => (
              <SortableHeader column={column} title="Services" />
            ),
          },
        ]}
      />
    </Section>
  );
}
