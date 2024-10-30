import { Section } from "@components/layouts";
import {
  container_state_intention,
  stroke_color_class_by_intention,
} from "@lib/color";
import { useRead } from "@lib/hooks";
import { cn } from "@lib/utils";
import { DataTable, SortableHeader } from "@ui/data-table";
import { useStack } from ".";
import { Types } from "komodo_client";
import { Fragment, ReactNode } from "react";
import { Link } from "react-router-dom";
import { Button } from "@ui/button";
import { Layers2 } from "lucide-react";
import { DockerResourceLink, StatusBadge } from "@components/util";

export const StackServices = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) => {
  const info = useStack(id)?.info;
  const server_id = info?.server_id;
  const state = info?.state ?? Types.StackState.Unknown;
  const services = useRead(
    "ListStackServices",
    { stack: id },
    { refetchInterval: 10_000 }
  ).data;
  if (
    !services ||
    services.length === 0 ||
    [Types.StackState.Unknown, Types.StackState.Down].includes(state)
  ) {
    return null;
  }
  return (
    <Section titleOther={titleOther}>
      <div className="lg:min-h-[300px]">
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
              cell: ({ row }) => {
                const state = row.original.container?.state;
                const color = stroke_color_class_by_intention(
                  container_state_intention(state)
                );
                return (
                  <Link
                    to={`/stacks/${id}/service/${row.original.service}`}
                    onClick={(e) => e.stopPropagation()}
                  >
                    <Button
                      variant="link"
                      className="flex gap-2 items-center p-0"
                    >
                      <Layers2 className={cn("w-4 h-4", color)} />
                      {row.original.service}
                    </Button>
                  </Link>
                );
              },
              // size: 200,
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
              size: 300,
              header: ({ column }) => (
                <SortableHeader column={column} title="Networks" />
              ),
              cell: ({ row }) => (
                <div className="flex items-center gap-2 flex-wrap">
                  {server_id &&
                    row.original.container?.networks.map((network, i) => (
                      <Fragment key={network}>
                        <DockerResourceLink
                          type="network"
                          server_id={server_id}
                          name={network}
                        />
                        {i !== row.original.container!.networks.length - 1 && (
                          <div className="text-muted-foreground">|</div>
                        )}
                      </Fragment>
                    ))}
                </div>
              ),
              // size: 200,
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
              // size: 120,
            },
          ]}
        />
      </div>
    </Section>
  );
};
