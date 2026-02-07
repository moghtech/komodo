import { ContainerPort } from "@/components/docker/container-ports";
import DockerResourceLink from "@/components/docker/link";
import ResourceUpdates from "@/components/updates/resource";
import { containerStateIntention, hexColorByIntention } from "@/lib/color";
import {
  useContainerPortsMap,
  usePermissions,
  useRead,
  useSetTitle,
} from "@/lib/hooks";
import { UsableResource } from "@/resources";
import { ResourceDescription, ResourceLink } from "@/resources/common";
import { ServerComponents } from "@/resources/server";
import { ICONS } from "@/theme/icons";
import DividedChildren from "@/ui/divided-children";
import EntityHeader from "@/ui/entity-header";
import EntityPage from "@/ui/entity-page";
import Section from "@/ui/section";
import { Center, Group, Stack, Text } from "@mantine/core";
import { Types } from "komodo_client";
import { useParams } from "react-router-dom";
import { ContainerExecutions } from "./executions";
import { DataTable } from "@/ui/data-table";
import DockerLabelsSection from "@/components/docker/labels-section";

export default function Container() {
  const {
    type,
    id: serverId,
    container: containerName,
  } = useParams() as {
    type: string;
    id: string;
    container: string;
  };
  if (type !== "servers") {
    return (
      <Center h="50vh">
        <Text>This resource type does not have any containers.</Text>
      </Center>
    );
  }
  return <ContainerInner serverId={serverId} containerName={containerName} />;
}

function ContainerInner({
  serverId,
  containerName,
}: {
  serverId: string;
  containerName: string;
}) {
  const server = ServerComponents.useListItem(serverId);
  useSetTitle(`${server?.name} | Container | ${containerName}`);
  const { canExecute } = usePermissions({ type: "Server", id: serverId });
  const listContainer = useRead(
    "ListDockerContainers",
    {
      server: serverId,
    },
    { refetchInterval: 10_000 },
  ).data?.find((container) => container.name === containerName);
  const inspect = useRead("InspectDockerContainer", {
    server: serverId,
    container: containerName,
  }).data;
  const { data: attached } = useRead(
    "GetResourceMatchingContainer",
    { server: serverId, container: containerName },
    { refetchInterval: 10_000 },
  );
  const portsMap = useContainerPortsMap(listContainer?.ports ?? []);

  const state = listContainer?.state ?? Types.ContainerStateStatusEnum.Empty;
  const intention = containerStateIntention(state);

  const Header = (
    <Stack justify="space-between">
      <Stack
        gap="md"
        pb="md"
        bd="1px solid var(--mantine-color-accent-border-0)"
        bdrs="md"
      >
        <EntityHeader
          name={listContainer?.name}
          icon={
            <ICONS.Container
              size="2rem"
              color={hexColorByIntention(intention)}
            />
          }
          intent={intention}
          state={state}
          status={listContainer?.status}
        />
        <DividedChildren px="md">
          <ResourceLink type="Server" id={serverId} />
          {attached?.resource && (
            <ResourceLink
              type={attached.resource.type as UsableResource}
              id={attached.resource.id}
            />
          )}
          {listContainer?.image && (
            <DockerResourceLink
              type="image"
              serverId={serverId}
              name={listContainer.image}
              id={listContainer.image_id}
            />
          )}
          {listContainer?.networks?.map((network) => (
            <DockerResourceLink
              key={network}
              type="network"
              serverId={serverId}
              name={network}
            />
          ))}
          {listContainer?.volumes?.map((volume) => (
            <DockerResourceLink
              key={volume}
              type="volume"
              serverId={serverId}
              name={volume}
            />
          ))}
          {Object.entries(portsMap).map(([hostPort, ports]) => (
            <ContainerPort
              key={hostPort}
              hostPort={hostPort}
              ports={ports}
              serverId={serverId}
            />
          ))}
        </DividedChildren>
      </Stack>
      <ResourceDescription type="Server" id={serverId} />
    </Stack>
  );

  return (
    <EntityPage
      backTo={"/servers/" + serverId}
      actions={<>{/* <NewDeployment id={id} name={container_name} /> */}</>}
    >
      <Stack hiddenFrom="xl" w="100%">
        {Header}
        <ResourceUpdates type="Server" id={serverId} />
      </Stack>
      <Group
        visibleFrom="xl"
        gap="xl"
        w="100%"
        align="stretch"
        grow
        preventGrowOverflow={false}
      >
        {Header}
        <ResourceUpdates type="Server" id={serverId} />
      </Group>

      <Stack mt="lg" gap="xl">
        {canExecute && (
          <Section
            title="Execute"
            icon={<ICONS.Execution size="1.3rem" />}
            my="xl"
          >
            <Group>
              {Object.entries(ContainerExecutions).map(([key, Execution]) => (
                <Execution
                  key={key}
                  serverId={serverId}
                  container={containerName}
                />
              ))}
            </Group>
          </Section>
        )}

        {/* TOP LEVEL CONTAINER INFO */}
        {listContainer && (
          <Section title="Details" icon={<ICONS.Info size="1.3rem" />}>
            <DataTable
              tableKey="container-info"
              data={[listContainer]}
              columns={[
                {
                  header: "Id",
                  accessorKey: "id",
                },
                {
                  header: "Image",
                  accessorKey: "image",
                },
                {
                  header: "Network Mode",
                  accessorKey: "network_mode",
                },
                {
                  header: "Networks",
                  accessorKey: "networks",
                },
              ]}
            />
          </Section>
        )}

        <DockerLabelsSection labels={inspect?.Config?.Labels} />
      </Stack>
    </EntityPage>
  );
}
