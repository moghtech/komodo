import { Section } from "@components/layouts";
import { container_state_intention, swarm_state_intention } from "@lib/color";
import { useRead } from "@lib/hooks";
import { DataTable, SortableHeader } from "@ui/data-table";
import { useStack } from ".";
import { Types } from "komodo_client";
import { Fragment, ReactNode } from "react";
import {
  ContainerPortsTableView,
  DockerResourceLink,
  StackServiceLink,
  StatusBadge,
} from "@components/util";

export const StackServices = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) => {
  const info = useStack(id)?.info;
  const state = info?.state ?? Types.StackState.Unknown;
  const services = useRead(
    "ListStackServices",
    { stack: id },
    { refetchInterval: 10_000 }
  ).data;
  console.log(services);
  if (
    !services ||
    services.length === 0 ||
    [Types.StackState.Unknown, Types.StackState.Down].includes(state)
  ) {
    return <Section titleOther={titleOther}>No Services Available</Section>;
  }
  return (
    <Section titleOther={titleOther}>
      <div className="lg:min-h-[300px]">
        {info?.swarm_id ? (
          <StackServicesSwarm stack_id={id} services={services} />
        ) : info?.server_id ? (
          <StackServicesServer
            stack_id={id}
            server_id={info.server_id}
            services={services}
          />
        ) : (
          <></>
        )}
      </div>
    </Section>
  );
};

const StackServicesSwarm = ({
  stack_id,
  services,
}: {
  stack_id: string;
  services: Types.ListStackServicesResponse;
}) => {
  return (
    <DataTable
      tableKey="StackServices"
      data={services}
      columns={[
        {
          accessorKey: "service",
          size: 200,
          header: ({ column }) => (
            <SortableHeader column={column} title="Service" />
          ),
          cell: ({ row }) => (
            <StackServiceLink id={stack_id} service={row.original.service} />
          ),
        },
        {
          accessorKey: "swarm_service.State",
          size: 160,
          header: ({ column }) => (
            <SortableHeader column={column} title="State" />
          ),
          cell: ({ row }) => {
            const state = row.original.swarm_service?.State;
            return (
              <StatusBadge text={state} intent={swarm_state_intention(state)} />
            );
          },
        },
        {
          accessorKey: "swarm_service.Runtime",
          size: 300,
          header: ({ column }) => (
            <SortableHeader column={column} title="Runtime" />
          ),
        },
        {
          accessorKey: "swarm_service.Image",
          size: 300,
          header: ({ column }) => (
            <SortableHeader column={column} title="Image" />
          ),
          cell: ({ row }) => {
            // It usually returns the image hash after the @, its very long so removed here
            return row.original.swarm_service?.Image?.split("@")?.[0];
          },
        },
        {
          accessorKey: "swarm_service.Replicas",
          size: 300,
          header: ({ column }) => (
            <SortableHeader column={column} title="Replicas" />
          ),
        },
      ]}
    />
  );
};

const StackServicesServer = ({
  stack_id,
  server_id,
  services,
}: {
  stack_id: string;
  server_id: string;
  services: Types.ListStackServicesResponse;
}) => {
  return (
    <DataTable
      tableKey="StackServices"
      data={services}
      columns={[
        {
          accessorKey: "service",
          size: 200,
          header: ({ column }) => (
            <SortableHeader column={column} title="Service" />
          ),
          cell: ({ row }) => (
            <StackServiceLink id={stack_id} service={row.original.service} />
          ),
        },
        {
          accessorKey: "container.state",
          size: 160,
          header: ({ column }) => (
            <SortableHeader column={column} title="State" />
          ),
          cell: ({ row }) => {
            const state = row.original.container?.state;
            return (
              <StatusBadge
                text={state}
                intent={container_state_intention(state)}
              />
            );
          },
        },
        {
          accessorKey: "container.image",
          size: 300,
          header: ({ column }) => (
            <SortableHeader column={column} title="Image" />
          ),
          cell: ({ row }) =>
            server_id && (
              <DockerResourceLink
                type="image"
                server_id={server_id}
                name={row.original.container?.image}
                id={row.original.container?.image_id}
              />
            ),
          // size: 200,
        },
        {
          accessorKey: "container.networks.0",
          size: 200,
          header: ({ column }) => (
            <SortableHeader column={column} title="Networks" />
          ),
          cell: ({ row }) =>
            (row.original.container?.networks?.length ?? 0) > 0 ? (
              <div className="flex items-center gap-2 flex-wrap">
                {server_id &&
                  row.original.container?.networks?.map((network, i) => (
                    <Fragment key={network}>
                      <DockerResourceLink
                        type="network"
                        server_id={server_id}
                        name={network}
                      />
                      {i !== row.original.container!.networks!.length - 1 && (
                        <div className="text-muted-foreground">|</div>
                      )}
                    </Fragment>
                  ))}
              </div>
            ) : (
              server_id &&
              row.original.container?.network_mode && (
                <DockerResourceLink
                  type="network"
                  server_id={server_id}
                  name={row.original.container.network_mode}
                />
              )
            ),
        },
        {
          accessorKey: "container.ports.0",
          size: 200,
          header: ({ column }) => (
            <SortableHeader column={column} title="Ports" />
          ),
          cell: ({ row }) => (
            <ContainerPortsTableView
              ports={row.original.container?.ports ?? []}
              server_id={server_id}
            />
          ),
        },
      ]}
    />
  );
};
