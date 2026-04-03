import { filterMultitermBySplit } from "@/lib/utils";
import { ICONS } from "@/lib/icons";
import { DataTable, SortableHeader } from "mogh_ui";
import { Section, SectionProps } from "mogh_ui";
import { ShowHideButton } from "mogh_ui";
import { Group } from "@mantine/core";
import { Types } from "komodo_client";
import SwarmResourceLink from "./link";
import { useRead } from "@/lib/hooks";
import { swarmTaskStateIntention } from "@/lib/color";
import { StatusBadge } from "mogh_ui";
import { SearchInput } from "mogh_ui";

export interface SwarmTasksSectionProps extends SectionProps {
  id: string;
  tasks: Types.SwarmTaskListItem[];
  show?: boolean;
  setShow?: (show: boolean) => void;
  _search: [string, (search: string) => void];
}

export default function SwarmTasksSection({
  id,
  tasks: _tasks,
  show = true,
  setShow,
  titleOther,
  _search,
  ...sectionProps
}: SwarmTasksSectionProps) {
  const nodes =
    useRead("ListSwarmNodes", { swarm: id }, { refetchInterval: 10_000 })
      .data ?? [];
  const services =
    useRead("ListSwarmServices", { swarm: id }, { refetchInterval: 10_000 })
      .data ?? [];

  const tasks = _tasks.map((task) => {
    return {
      ...task,
      node: nodes.find((node) => task.NodeID === node.ID),
      service: services.find((service) => task.ServiceID === service.ID),
    };
  });

  const filtered = filterMultitermBySplit(tasks, _search[0], (task) => [
    task.ID,
    task.node?.Hostname,
    task.service?.Name,
  ]);

  return (
    <Section
      titleOther={titleOther}
      title={!titleOther ? "Tasks" : undefined}
      icon={!titleOther ? <ICONS.SwarmTask size="1.3rem" /> : undefined}
      actions={
        _search || setShow ? (
          <Group>
            {_search && (
              <SearchInput value={_search[0]} onSearch={_search[1]} />
            )}
            {setShow && <ShowHideButton show={show} setShow={setShow} />}
          </Group>
        ) : undefined
      }
      {...sectionProps}
    >
      {show && (
        <DataTable
          tableKey="swarm-tasks"
          data={filtered}
          columns={[
            {
              accessorKey: "ID",
              header: ({ column }) => (
                <SortableHeader column={column} title="ID" />
              ),
              cell: ({ row }) => (
                <SwarmResourceLink
                  type="Task"
                  swarmId={id}
                  resourceId={row.original.ID}
                  name={row.original.ID}
                />
              ),
              size: 150,
            },
            {
              accessorKey: "node.Hostname",
              header: ({ column }) => (
                <SortableHeader column={column} title="Node" />
              ),
              cell: ({ row }) => (
                <SwarmResourceLink
                  type="Node"
                  swarmId={id}
                  resourceId={row.original.node?.ID}
                  name={row.original.node?.Hostname}
                />
              ),
              size: 200,
            },
            {
              accessorKey: "service.Name",
              header: ({ column }) => (
                <SortableHeader column={column} title="Service" />
              ),
              cell: ({ row }) => (
                <SwarmResourceLink
                  type="Service"
                  swarmId={id}
                  resourceId={row.original.service?.ID}
                  name={row.original.service?.Name}
                />
              ),
              size: 200,
            },
            {
              accessorKey: "State",
              header: ({ column }) => (
                <SortableHeader column={column} title="State" />
              ),
              cell: ({ row }) => (
                <StatusBadge
                  text={row.original.State}
                  intent={swarmTaskStateIntention(
                    row.original.State,
                    row.original.DesiredState,
                  )}
                />
              ),
            },
            {
              accessorKey: "DesiredState",
              header: ({ column }) => (
                <SortableHeader column={column} title="Desired State" />
              ),
              cell: ({ row }) => (
                <StatusBadge
                  text={row.original.DesiredState}
                  intent={swarmTaskStateIntention(
                    row.original.State,
                    row.original.DesiredState,
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
      )}
    </Section>
  );
}
