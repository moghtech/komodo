import { DataTable, SortableHeader } from "@ui/data-table";
import { ResourceLink } from "../common";
import { TableTags } from "@components/tags";
import { SwarmComponents, SwarmLink } from ".";
import { Types } from "komodo_client";
import { useRead, useSelectedResources } from "@lib/hooks";
import { Dispatch, ReactNode, SetStateAction } from "react";
import { filterBySplit } from "@lib/utils";
import { Section } from "@components/layouts";
import { Search } from "lucide-react";
import { Input } from "@ui/input";

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
              <SwarmLink
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

export const SwarmStackServicesTable = ({
  id,
  services,
  titleOther,
  _search,
}: {
  id: string;
  services: Types.SwarmStackServiceListItem[];
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
              <SwarmLink
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
            size: 200,
          },
          {
            accessorKey: "Image",
            header: ({ column }) => (
              <SortableHeader column={column} title="Image" />
            ),
            size: 200,
          },
          {
            accessorKey: "Mode",
            header: ({ column }) => (
              <SortableHeader column={column} title="Mode" />
            ),
          },
          {
            accessorKey: "Replicas",
            header: ({ column }) => (
              <SortableHeader column={column} title="Replicas" />
            ),
          },
          {
            accessorKey: "Ports",
            header: ({ column }) => (
              <SortableHeader column={column} title="Ports" />
            ),
          },
        ]}
      />
    </Section>
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

  const filtered = filterBySplit(
    tasks,
    search,
    (task) =>
      task.Name ?? task.service?.Name ?? task.node?.Hostname ?? "Unknown"
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
            accessorKey: "ID",
            header: ({ column }) => (
              <SortableHeader column={column} title="Id" />
            ),
            cell: ({ row }) => (
              <SwarmLink
                type="Task"
                swarm_id={id}
                resource_id={row.original.ID}
                name={row.original.ID}
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
              <SwarmLink
                type="Service"
                swarm_id={id}
                resource_id={row.original.service?.ID}
                name={row.original.service?.Name}
              />
            ),
            size: 200,
          },
          {
            accessorKey: "node.Hostname",
            header: ({ column }) => (
              <SortableHeader column={column} title="Node" />
            ),
            cell: ({ row }) => (
              <SwarmLink
                type="Node"
                swarm_id={id}
                resource_id={row.original.node?.ID}
                name={row.original.node?.Hostname}
              />
            ),
            size: 200,
          },
          {
            accessorKey: "State",
            header: ({ column }) => (
              <SortableHeader column={column} title="State" />
            ),
          },
          {
            accessorKey: "DesiredState",
            header: ({ column }) => (
              <SortableHeader column={column} title="Desired State" />
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
};

export const SwarmStackTasksTable = ({
  id,
  tasks: _tasks,
  titleOther,
  _search,
}: {
  id: string;
  tasks: Types.SwarmStackTaskListItem[];
  titleOther: ReactNode;
  _search: [string, Dispatch<SetStateAction<string>>];
}) => {
  const [search, setSearch] = _search;

  const nodes =
    useRead("ListSwarmNodes", { swarm: id }, { refetchInterval: 10_000 })
      .data ?? [];
  const tasks = _tasks.map((task) => {
    return {
      ...task,
      node: nodes.find(
        (node) =>
          (task.Node ?? false) &&
          (task.Node === node.ID ||
            task.Node === node.Hostname ||
            task.Node === node.Name)
      ),
    };
  });

  const filtered = filterBySplit(
    tasks,
    search,
    (task) => task.Name ?? task.node?.Hostname ?? "Unknown"
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
        tableKey="swarm-tasks"
        data={filtered}
        columns={[
          {
            accessorKey: "ID",
            header: ({ column }) => (
              <SortableHeader column={column} title="Id" />
            ),
            cell: ({ row }) => (
              <SwarmLink
                type="Task"
                swarm_id={id}
                resource_id={row.original.ID}
                name={row.original.ID}
              />
            ),
            size: 200,
          },
          {
            accessorKey: "node.Hostname",
            header: ({ column }) => (
              <SortableHeader column={column} title="Node" />
            ),
            cell: ({ row }) => (
              <SwarmLink
                type="Node"
                swarm_id={id}
                resource_id={row.original.node?.ID}
                name={row.original.node?.Hostname}
              />
            ),
            size: 200,
          },
          {
            accessorKey: "Image",
            header: ({ column }) => (
              <SortableHeader column={column} title="Image" />
            ),
          },
          {
            accessorKey: "CurrentState",
            header: ({ column }) => (
              <SortableHeader column={column} title="State" />
            ),
          },
          {
            accessorKey: "DesiredState",
            header: ({ column }) => (
              <SortableHeader column={column} title="Desired State" />
            ),
          },
        ]}
      />
    </Section>
  );
};
