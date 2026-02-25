import { ContainerPort } from "@/components/docker/container-ports";
import DockerResourceLink from "@/components/docker/link";
import { containerStateIntention } from "@/lib/color";
import {
  useContainerPortsMap,
  usePermissions,
  useRead,
  useSetTitle,
} from "@/lib/hooks";
import { UsableResource } from "@/resources";
import { useServer } from "@/resources/server";
import { ICONS } from "@/theme/icons";
import Section from "@/ui/section";
import { Center, Text } from "@mantine/core";
import { Types } from "komodo_client";
import { useParams } from "react-router-dom";
import { ContainerExecutions } from "./executions";
import { DataTable } from "@/ui/data-table";
import DockerLabelsSection from "@/components/docker/labels-section";
import ContainerTabs from "./tabs";
import ResourceLink from "@/resources/link";
import ResourceSubPage from "@/resources/sub-page";

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
  const server = useServer(serverId);
  useSetTitle(`${server?.name} | Container | ${containerName}`);
  const { specific } = usePermissions({
    type: "Server",
    id: serverId,
  });
  const listContainer = useRead(
    "ListDockerContainers",
    {
      server: serverId,
    },
    { refetchInterval: 10_000 },
  ).data?.find((container) => container.name === containerName);
  const inspect = useRead(
    "InspectDockerContainer",
    {
      server: serverId,
      container: containerName,
    },
    { enabled: specific.includes(Types.SpecificPermission.Inspect) },
  ).data;
  const { data: attached } = useRead(
    "GetResourceMatchingContainer",
    { server: serverId, container: containerName },
    { refetchInterval: 10_000 },
  );
  const portsMap = useContainerPortsMap(listContainer?.ports ?? []);

  const state = listContainer?.state ?? Types.ContainerStateStatusEnum.Empty;
  const intention = containerStateIntention(state);

  return (
    <ResourceSubPage
      entityTypeName="Container"
      parentType="Server"
      parentId={serverId}
      name={listContainer?.name}
      icon={ICONS.Container}
      intent={intention}
      state={state}
      status={listContainer?.status}
      info={
        <>
          {attached?.resource && (
            <ResourceLink
              type={attached.resource.type as UsableResource}
              id={attached.resource.id}
            />
          )}
          {listContainer?.image && (
            <DockerResourceLink
              type="Image"
              serverId={serverId}
              name={listContainer.image}
              id={listContainer.image_id}
            />
          )}
          {listContainer?.networks?.map((network) => (
            <DockerResourceLink
              key={network}
              type="Network"
              serverId={serverId}
              name={network}
            />
          ))}
          {listContainer?.volumes?.map((volume) => (
            <DockerResourceLink
              key={volume}
              type="Volume"
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
        </>
      }
      executions={
        <>
          {Object.entries(ContainerExecutions).map(([key, Execution]) => (
            <Execution
              key={key}
              serverId={serverId}
              container={containerName}
            />
          ))}
        </>
      }
    >
      <ContainerTabs
        server={serverId}
        container={containerName}
        state={state}
        inspect={inspect}
      />

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
    </ResourceSubPage>
  );
}
