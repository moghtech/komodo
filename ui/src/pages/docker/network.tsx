import { useExecute, usePermissions, useRead, useSetTitle } from "@/lib/hooks";
import DockerLabelsSection from "@/components/docker/labels-section";
import DockerResourceLink from "@/components/docker/link";
import DockerOptions from "@/components/docker/options";
import InspectSection from "@/components/inspect-section";
import { useServer } from "@/resources/server";
import ResourceSubPage from "@/resources/sub-page";
import { ICONS } from "@/lib/icons";
import { ConfirmButton } from "mogh_ui";
import { DataTable, SortableHeader } from "mogh_ui";
import { Section } from "mogh_ui";
import { Center, Group, Loader, Text } from "@mantine/core";
import { Types } from "komodo_client";
import { Waypoints } from "lucide-react";
import { useNavigate, useParams } from "react-router-dom";

export default function Network() {
  const { type, id, network } = useParams() as {
    type: string;
    id: string;
    network: string;
  };
  if (type !== "servers") {
    return (
      <Center h="50vh">
        <Text>This resource type does not have any networks.</Text>
      </Center>
    );
  }
  return <NetworkInner serverId={id} networkName={network} />;
}

function NetworkInner({
  serverId,
  networkName,
}: {
  serverId: string;
  networkName: string;
}) {
  const server = useServer(serverId);
  useSetTitle(`${server?.name} | Network | ${networkName}`);
  const nav = useNavigate();

  const { specific } = usePermissions({
    type: "Server",
    id: serverId,
  });

  const {
    data: network,
    isPending,
    isError,
  } = useRead("InspectDockerNetwork", {
    server: serverId,
    network: networkName,
  });

  const { mutate: deleteNetwork, isPending: deletePending } = useExecute(
    "DeleteNetwork",
    {
      onSuccess: () => nav("/servers/" + serverId),
    },
  );

  if (isPending) {
    return (
      <Center h="30vh">
        <Loader size="xl" />
      </Center>
    );
  }

  if (isError) {
    return (
      <Center h="30vh">
        <Text>Failed to inspect network.</Text>
      </Center>
    );
  }

  if (!network) {
    return (
      <Center h="30vh">
        <Text>No network found with given name: {networkName}</Text>
      </Center>
    );
  }

  const containers = Object.values(network.Containers ?? {});
  const ipamDriver = network.IPAM?.Driver;
  const ipamConfig =
    network.IPAM?.Config.map((config) => ({
      ...config,
      Driver: ipamDriver,
    })) ?? [];

  const unused =
    !["none", "host", "bridge"].includes(networkName) &&
    containers &&
    containers.length === 0
      ? true
      : false;

  const intention = unused ? "Critical" : "Good";

  return (
    <ResourceSubPage
      entityTypeName="Network"
      parentType="Server"
      parentId={serverId}
      name={networkName}
      icon={ICONS.Network}
      intent={intention}
      state={unused ? "Unused" : "In Use"}
      info={
        <>
          <Group gap="xs">
            <Text c="dimmed">IPV6:</Text>
            <Text>{network.EnableIPv6 ? "Enabled" : "Disabled"}</Text>
          </Group>
          {network.Id && (
            <Group gap="xs">
              <Text c="dimmed">Id:</Text>
              <Text title={network.Id} maw={150} className="text-ellipsis">
                {network.Id}
              </Text>
            </Group>
          )}
        </>
      }
      executions={
        unused ? (
          <ConfirmButton
            variant="filled"
            color="red"
            icon={<ICONS.Delete size="1rem" />}
            loading={deletePending}
            onClick={() =>
              deleteNetwork({ server: serverId, name: networkName })
            }
          >
            Delete Network
          </ConfirmButton>
        ) : undefined
      }
    >
      {containers && containers.length > 0 && (
        <Section title="Containers" icon={<ICONS.Container size="1.3rem" />}>
          <DataTable
            tableKey="network-containers"
            data={containers}
            columns={[
              {
                accessorKey: "Name",
                header: ({ column }) => (
                  <SortableHeader column={column} title="Name" />
                ),
                cell: ({ row }) =>
                  row.original.Name ? (
                    <DockerResourceLink
                      type="Container"
                      serverId={serverId}
                      name={row.original.Name}
                    />
                  ) : (
                    "Unknown"
                  ),
                size: 200,
              },
              {
                accessorKey: "IPv4Address",
                header: ({ column }) => (
                  <SortableHeader column={column} title="IPv4" />
                ),
                cell: ({ row }) => row.original.IPv4Address || "None",
              },
              {
                accessorKey: "IPv6Address",
                header: ({ column }) => (
                  <SortableHeader column={column} title="IPv6" />
                ),
                cell: ({ row }) => row.original.IPv6Address || "None",
              },
              {
                accessorKey: "MacAddress",
                header: ({ column }) => (
                  <SortableHeader column={column} title="Mac" />
                ),
                cell: ({ row }) => row.original.MacAddress || "None",
              },
            ]}
          />
        </Section>
      )}

      {/* TOP LEVEL NETWORK INFO */}
      <Section title="Details" icon={<ICONS.Info size="1.3rem" />}>
        <DataTable
          tableKey="network-info"
          data={[network]}
          columns={[
            {
              accessorKey: "Driver",
              header: "Driver",
            },
            {
              accessorKey: "Scope",
              header: "Scope",
            },
            {
              accessorKey: "Attachable",
              header: "Attachable",
            },
            {
              accessorKey: "Internal",
              header: "Internal",
            },
          ]}
        />
        {network.Options && (
          <DockerOptions options={Object.entries(network.Options)} />
        )}
      </Section>

      {ipamConfig.length > 0 && (
        <Section title="IPAM" icon={<Waypoints size="1.3rem" />}>
          <DataTable
            tableKey="network-ipam"
            data={ipamConfig}
            columns={[
              {
                accessorKey: "Driver",
                header: "Driver",
              },
              {
                accessorKey: "Subnet",
                header: "Subnet",
              },
              {
                accessorKey: "Gateway",
                header: "Gateway",
              },
              {
                accessorKey: "IPRange",
                header: "IPRange",
              },
            ]}
          />
          {network.IPAM?.Options && (
            <DockerOptions options={Object.entries(network.IPAM.Options)} />
          )}
        </Section>
      )}

      {specific.includes(Types.SpecificPermission.Inspect) && (
        <InspectSection json={network} showToggle />
      )}

      <DockerLabelsSection labels={network?.Labels} />
    </ResourceSubPage>
  );
}
