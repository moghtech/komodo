import { useRead } from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import { Prune } from "@/resources/server/executions";
import { ICONS } from "@/theme/icons";
import { DataTable, SortableHeader } from "@/ui/data-table";
import Section, { SectionProps } from "@/ui/section";
import ShowHideButton from "@/ui/show-hide-button";
import { Group, TextInput } from "@mantine/core";
import { Types } from "komodo_client";
import { Dispatch, SetStateAction } from "react";
import DockerResourceLink from "./link";
import StatusBadge from "@/ui/status-badge";
import { containerStateIntention } from "@/lib/color";
import DividedChildren from "@/ui/divided-children";
import { ContainerPorts } from "./container-ports";

export interface ContainersSectionProps extends SectionProps {
  serverId: string;
  containers: Types.ListDockerContainersResponse;
  show?: boolean;
  setShow?: (show: boolean) => void;
  pruneButton?: boolean;
  forceTall?: boolean;
  _search?: [string, Dispatch<SetStateAction<string>>];
}

export default function ContainersSection({
  serverId,
  containers,
  show = true,
  setShow,
  pruneButton,
  forceTall,
  _search,
  titleOther,
  ...props
}: ContainersSectionProps) {
  const allRunning = useRead("ListDockerContainers", {
    server: serverId,
  }).data?.every(
    (container) => container.state === Types.ContainerStateStatusEnum.Running,
  );
  const filtered = _search
    ? filterBySplit(containers, _search[0], (container) => container.name)
    : containers;

  return (
    <Section
      titleOther={titleOther}
      title={!titleOther ? "Containers" : undefined}
      icon={!titleOther ? <ICONS.Container size="1.3rem" /> : undefined}
      actions={
        (pruneButton && !allRunning) || _search ? (
          <Group>
            {pruneButton && !allRunning && (
              <Prune serverId={serverId} type="Containers" />
            )}
            {_search && (
              <TextInput
                placeholder="search..."
                leftSection={<ICONS.Search size="1rem" />}
                value={_search[0]}
                onChange={(e) => _search[1](e.target.value)}
                w={{ base: 200, lg: 300 }}
              />
            )}
            {setShow && <ShowHideButton show={show} setShow={setShow} />}
          </Group>
        ) : undefined
      }
      {...props}
    >
      {show && (
        <DataTable
          mih={forceTall ? "60vh" : undefined}
          tableKey="server-containers"
          data={filtered}
          columns={[
            {
              accessorKey: "name",
              size: 260,
              header: ({ column }) => (
                <SortableHeader column={column} title="Name" />
              ),
              cell: ({ row }) => (
                <DockerResourceLink
                  type="Container"
                  serverId={serverId}
                  name={row.original.name}
                />
              ),
            },
            {
              accessorKey: "state",
              size: 160,
              header: ({ column }) => (
                <SortableHeader column={column} title="State" />
              ),
              cell: ({ row }) => {
                const state = row.original?.state;
                return (
                  <StatusBadge
                    text={state}
                    intent={containerStateIntention(state)}
                  />
                );
              },
            },
            {
              accessorKey: "image",
              size: 300,
              header: ({ column }) => (
                <SortableHeader column={column} title="Image" />
              ),
              cell: ({ row }) => (
                <DockerResourceLink
                  type="Image"
                  serverId={serverId}
                  name={row.original.image}
                  id={row.original.image_id}
                />
              ),
            },
            {
              accessorKey: "networks.0",
              size: 200,
              header: ({ column }) => (
                <SortableHeader column={column} title="Networks" />
              ),
              cell: ({ row }) =>
                (row.original.networks?.length ?? 0) > 0 ? (
                  <DividedChildren>
                    {row.original.networks?.map((network) => (
                      <DockerResourceLink
                        key={network}
                        type="Network"
                        serverId={serverId}
                        name={network}
                      />
                    ))}
                  </DividedChildren>
                ) : (
                  row.original.network_mode && (
                    <DockerResourceLink
                      type="Network"
                      serverId={serverId}
                      name={row.original.network_mode}
                    />
                  )
                ),
            },
            {
              accessorKey: "ports.0",
              size: 200,
              sortingFn: (a, b) => {
                const getMinHostPort = (row: typeof a) => {
                  const ports = row.original.ports ?? [];
                  if (!ports.length) return Number.POSITIVE_INFINITY;
                  const nums = ports
                    .map((p) => p.PublicPort)
                    .filter((p): p is number => typeof p === "number")
                    .map((n) => Number(n));
                  if (!nums.length || nums.some((n) => Number.isNaN(n))) {
                    return Number.POSITIVE_INFINITY;
                  }
                  return Math.min(...nums);
                };
                const pa = getMinHostPort(a);
                const pb = getMinHostPort(b);
                return pa === pb ? 0 : pa > pb ? 1 : -1;
              },
              header: ({ column }) => (
                <SortableHeader column={column} title="Ports" />
              ),
              cell: ({ row }) => (
                <ContainerPorts
                  ports={row.original.ports ?? []}
                  serverId={row.original.server_id}
                />
              ),
            },
            {
              accessorKey: "volumes.0",
              size: 200,
              header: ({ column }) => (
                <SortableHeader column={column} title="Volumes" />
              ),
              cell: ({ row }) => (
                <DividedChildren>
                  {row.original.volumes?.map((volume) => (
                    <DockerResourceLink
                      key={volume}
                      type="Volume"
                      serverId={row.original.server_id!}
                      name={volume}
                    />
                  ))}
                </DividedChildren>
              ),
            },
          ]}
        />
      )}
    </Section>
  );
}
