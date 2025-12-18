import { DataTable, SortableHeader } from "@ui/data-table";
import { ResourceLink } from "../common";
import { TableTags } from "@components/tags";
import { SwarmComponents, SwarmResourceLink } from ".";
import { Types } from "komodo_client";
import { useRead, useSelectedResources } from "@lib/hooks";
import { Dispatch, ReactNode, SetStateAction } from "react";
import { filterBySplit, filterMultitermBySplit } from "@lib/utils";
import { Section } from "@components/layouts";
import { Search } from "lucide-react";
import { Input } from "@ui/input";
import { StatusBadge } from "@components/util";
import { swarm_task_state_intention } from "@lib/color";

export const SwarmTable = ({ swarms }: { swarms: Types.SwarmListItem[] }) => {
  const [_, setSelectedResources] = useSelectedResources("Swarm");

  return (
    <DataTable
      tableKey="swarms"
      data={swarms}
      selectOptions={{
        selectKey: ({ name }) => name,
        onSelect: setSelectedResources,
      }}
      columns={[
        {
          header: ({ column }) => (
            <SortableHeader column={column} title="Name" />
          ),
          accessorKey: "name",
          cell: ({ row }) => <ResourceLink type="Swarm" id={row.original.id} />,
          size: 200,
        },
        {
          header: ({ column }) => (
            <SortableHeader column={column} title="State" />
          ),
          accessorKey: "info.state",
          cell: ({ row }) => <SwarmComponents.State id={row.original.id} />,
          size: 120,
        },
        {
          header: "Tags",
          cell: ({ row }) => <TableTags tag_ids={row.original.tags} />,
        },
      ]}
    />
  );
};

export const SwarmServicesTable = ({
  id,
  services,
  titleOther,
  _search,
}: {
  id: string;
  services: Types.SwarmServiceListItem[];
  titleOther: ReactNode;
  _search: [string, Dispatch<SetStateAction<string>>];
}) => {
  const [search, setSearch] = _search;
  const filtered = filterBySplit(
    services,
    search,
    (service) => service.Name ?? service.ID ?? "Unknown"
  );
  return (
    <Section
      titleOther={titleOther}
      actions={
        <div className="flex items-center gap-4 flex-wrap">
          <div className="relative">
            <Search className="w-4 absolute top-[50%] left-3 -translate-y-[50%] text-muted-foreground" />
            <Input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="search..."
              className="pl-8 w-[200px] lg:w-[300px]"
            />
          </div>
        </div>
      }
    >
      <DataTable
        containerClassName="min-h-[60vh]"
        tableKey="swarm-services"
        data={filtered}
        columns={[
          {
            accessorKey: "Name",
            header: ({ column }) => (
              <SortableHeader column={column} title="Name" />
            ),
            cell: ({ row }) => (
              <SwarmResourceLink
                type="Service"
                swarm_id={id}
                resource_id={row.original.Name}
                name={row.original.Name}
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
};

export const SwarmServiceTasksTable = ({
  id,
  service_id,
  titleOther,
  _search,
}: {
  id: string;
  service_id: string | undefined;
  titleOther: ReactNode;
  _search: [string, Dispatch<SetStateAction<string>>];
}) => {
  const tasks =
    useRead(
      "ListSwarmTasks",
      { swarm: id },
      { enabled: !!service_id }
    ).data?.filter((task) => service_id && task.ServiceID === service_id) ?? [];
  return (
    <SwarmTasksTable
      id={id}
      tasks={tasks}
      titleOther={titleOther}
      _search={_search}
    />
  );
};

export const SwarmTasksTable = ({
  id,
  tasks: _tasks,
  titleOther,
  _search,
}: {
  id: string;
  tasks: Types.SwarmTaskListItem[];
  titleOther: ReactNode;
  _search: [string, Dispatch<SetStateAction<string>>];
}) => {
  const [search, setSearch] = _search;
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

  const filtered = filterMultitermBySplit(tasks, search, (task) => [
    task.ID,
    task.node?.Hostname,
    task.service?.Name,
  ]);

  return (
    <Section
      titleOther={titleOther}
      actions={
        <div className="flex items-center gap-4 flex-wrap">
          <div className="relative">
            <Search className="w-4 absolute top-[50%] left-3 -translate-y-[50%] text-muted-foreground" />
            <Input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="search..."
              className="pl-8 w-[200px] lg:w-[300px]"
            />
          </div>
        </div>
      }
    >
      <DataTable
        containerClassName="min-h-[60vh]"
        tableKey="swarm-services"
        data={filtered}
        columns={[
          {
            accessorKey: "ID",
            header: ({ column }) => (
              <SortableHeader column={column} title="Id" />
            ),
            cell: ({ row }) => (
              <SwarmResourceLink
                type="Task"
                swarm_id={id}
                resource_id={row.original.ID}
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
                swarm_id={id}
                resource_id={row.original.node?.ID}
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
                swarm_id={id}
                resource_id={row.original.service?.ID}
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
                intent={swarm_task_state_intention(
                  row.original.State,
                  row.original.DesiredState
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
                intent={swarm_task_state_intention(
                  row.original.State,
                  row.original.DesiredState
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
};
