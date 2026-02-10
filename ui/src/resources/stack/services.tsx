import { Fragment, ReactNode } from "react";
import { useStack } from ".";
import { useRead } from "@/lib/hooks";
import { Types } from "komodo_client";
import Section from "@/ui/section";
import { DataTable, SortableHeader } from "@/ui/data-table";
import StackServiceLink from "@/components/stack-service-link";
import StatusBadge from "@/ui/status-badge";
import { containerStateIntention, swarmStateIntention } from "@/lib/color";
import DockerResourceLink from "@/components/docker/link";
import { Group } from "@mantine/core";
import { ContainerPorts } from "@/components/docker/container-ports";

export default function StackServices({
  id,
  titleOther,
}: {
  id: string;
  titleOther?: ReactNode;
}) {
  const info = useStack(id)?.info;
  const state = info?.state ?? Types.StackState.Unknown;
  const services = useRead(
    "ListStackServices",
    { stack: id },
    { refetchInterval: 10_000 },
  ).data;

  if (
    !services ||
    services.length === 0 ||
    [Types.StackState.Unknown, Types.StackState.Down].includes(state)
  ) {
    // return <Section titleOther={titleOther}>No Services Available</Section>;
    return null;
  }

  return (
    <Section titleOther={titleOther} mb="md">
      {info?.swarm_id ? (
        <StackServicesSwarm stackId={id} services={services} />
      ) : info?.server_id ? (
        <StackServicesServer
          stackId={id}
          serverId={info.server_id}
          services={services}
        />
      ) : (
        <></>
      )}
    </Section>
  );
}

function StackServicesSwarm({
  stackId,
  services,
}: {
  stackId: string;
  services: Types.ListStackServicesResponse;
}) {
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
            <StackServiceLink id={stackId} service={row.original.service} />
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
              <StatusBadge text={state} intent={swarmStateIntention(state)} />
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
}

function StackServicesServer({
  stackId,
  serverId,
  services,
}: {
  stackId: string;
  serverId: string;
  services: Types.ListStackServicesResponse;
}) {
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
            <StackServiceLink id={stackId} service={row.original.service} />
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
                intent={containerStateIntention(state)}
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
            serverId && (
              <DockerResourceLink
                type="Image"
                serverId={serverId}
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
              <Group>
                {serverId &&
                  row.original.container?.networks?.map((network, i) => (
                    <Fragment key={network}>
                      <DockerResourceLink
                        type="Network"
                        serverId={serverId}
                        name={network}
                      />
                      {i !== row.original.container!.networks!.length - 1 && (
                        <div className="text-muted-foreground">|</div>
                      )}
                    </Fragment>
                  ))}
              </Group>
            ) : (
              serverId &&
              row.original.container?.network_mode && (
                <DockerResourceLink
                  type="Network"
                  serverId={serverId}
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
            <ContainerPorts
              ports={row.original.container?.ports ?? []}
              serverId={serverId}
            />
          ),
        },
      ]}
    />
  );
}
