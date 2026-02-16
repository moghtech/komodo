import { Center, Group, Stack, Text } from "@mantine/core";
import { useParams } from "react-router-dom";
import {
  DeployStack,
  DestroyStack,
  PauseUnpauseStack,
  PullStack,
  RestartStack,
  StartStopStack,
} from "@/resources/stack/executions";
import { useStack } from "@/resources/stack";
import {
  useContainerPortsMap,
  usePermissions,
  useRead,
  useSetTitle,
} from "@/lib/hooks";
import { Types } from "komodo_client";
import { containerStateIntention, swarmStateIntention } from "@/lib/color";
import EntityHeader from "@/ui/entity-header";
import { ICONS } from "@/theme/icons";
import DividedChildren from "@/ui/divided-children";
import ResourceLink from "@/resources/link";
import ResourceDescription from "@/resources/description";
import EntityPage from "@/ui/entity-page";
import ResourceUpdates from "@/components/updates/resource";
import Section from "@/ui/section";
import SwarmResourceLink from "@/components/swarm/link";
import DockerResourceLink from "@/components/docker/link";
import { ContainerPort } from "@/components/docker/container-ports";
import StackServiceTabs from "./tabs";

type IdServiceComponent = React.FC<{ id: string; service?: string }>;

const Executions: { [action: string]: IdServiceComponent } = {
  DeployStack,
  PullStack,
  RestartStack,
  PauseUnpauseStack,
  StartStopStack,
  DestroyStack,
};

export default function StackService() {
  const {
    type,
    id: stackId,
    service: serviceName,
  } = useParams() as {
    type: string;
    id: string;
    service: string;
  };
  if (type !== "stacks") {
    return (
      <Center h="50vh">
        <Text>This resource type does not have any services.</Text>
      </Center>
    );
  }
  return <StackServiceInner stackId={stackId} serviceName={serviceName} />;
}

function StackServiceInner({
  stackId,
  serviceName,
}: {
  stackId: string;
  serviceName: string;
}) {
  const stack = useStack(stackId);
  useSetTitle(`${stack?.name} | ${serviceName}`);
  const { canExecute } = usePermissions({
    type: "Stack",
    id: stackId,
  });
  const services = useRead("ListStackServices", { stack: stackId }).data;
  const service = services?.find((s) => s.service === serviceName);

  const container = service?.container;
  const swarmService = service?.swarm_service;

  const portsMap = useContainerPortsMap(container?.ports ?? []);

  const state = swarmService?.State
    ? swarmService?.State
    : (container?.state ?? Types.ContainerStateStatusEnum.Empty);

  const intention = swarmService?.State
    ? swarmStateIntention(swarmService.State)
    : containerStateIntention(
        container?.state ?? Types.ContainerStateStatusEnum.Empty,
      );

  const Header = (
    <Stack justify="space-between">
      <Stack gap="md" pb="md" className="bordered-light" bdrs="md">
        <EntityHeader
          name={serviceName}
          icon={ICONS.Service}
          intent={intention}
          state={state}
          status={
            swarmService
              ? `${swarmService.Replicas} Replica${swarmService.Replicas === 1 ? "" : "s"}`
              : container?.status
          }
        />
        <DividedChildren px="md">
          <ResourceLink type="Stack" id={stackId} />

          {/* SWARM ONLY */}
          {stack?.info.swarm_id && (
            <>
              <ResourceLink type="Swarm" id={stack.info.swarm_id} />
              {swarmService?.Name && (
                <SwarmResourceLink
                  type="Service"
                  swarmId={stack.info.swarm_id}
                  resourceId={swarmService.Name}
                  name={swarmService.Name}
                />
              )}
              {swarmService?.Configs.map((config) => (
                <SwarmResourceLink
                  key={config}
                  type="Config"
                  swarmId={stack.info.swarm_id}
                  resourceId={config}
                  name={config}
                />
              ))}
              {swarmService?.Secrets.map((secret) => (
                <SwarmResourceLink
                  key={secret}
                  type="Secret"
                  swarmId={stack.info.swarm_id}
                  resourceId={secret}
                  name={secret}
                />
              ))}
            </>
          )}

          {/* SERVER ONLY */}
          {!stack?.info.swarm_id && stack?.info.server_id && (
            <>
              <ResourceLink type="Server" id={stack.info.server_id} />
              {container?.name && (
                <DockerResourceLink
                  type="Container"
                  serverId={stack.info.server_id}
                  name={container.name}
                />
              )}
              {container?.image && (
                <DockerResourceLink
                  type="Image"
                  serverId={stack.info.server_id}
                  name={container.image}
                  id={container.image_id}
                />
              )}
              {container?.networks?.map((network) => (
                <DockerResourceLink
                  key={network}
                  type="Network"
                  serverId={stack.info.server_id}
                  name={network}
                />
              ))}
              {container?.volumes?.map((volume) => (
                <DockerResourceLink
                  key={volume}
                  type="Volume"
                  serverId={stack.info.server_id}
                  name={volume}
                />
              ))}
              {Object.keys(portsMap).map((hostPort) => (
                <ContainerPort
                  key={hostPort}
                  hostPort={hostPort}
                  ports={portsMap[hostPort]}
                  serverId={stack.info.server_id}
                />
              ))}
            </>
          )}
        </DividedChildren>
      </Stack>
      <ResourceDescription type="Stack" id={stackId} />
    </Stack>
  );

  return (
    <EntityPage backTo={"/stacks/" + stackId}>
      <Stack hiddenFrom="xl" w="100%">
        {Header}
        <ResourceUpdates type="Stack" id={stackId} />
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
        <ResourceUpdates type="Stack" id={stackId} />
      </Group>

      <Stack mt="lg" gap="xl">
        {canExecute && (
          <Section
            title="Execute"
            icon={<ICONS.Execution size="1.3rem" />}
            my="xl"
          >
            <Group>
              {Object.entries(Executions).map(([key, Execution]) => (
                <Execution key={key} id={stackId} service={serviceName} />
              ))}
            </Group>
          </Section>
        )}
      </Stack>

      {/** TABS */}
      {stack && (
        <StackServiceTabs
          stack={stack}
          service={serviceName}
          container={container}
          swarmService={swarmService}
          intention={intention}
        />
      )}
    </EntityPage>
  );
}
