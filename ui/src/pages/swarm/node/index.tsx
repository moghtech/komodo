import RemoveSwarmResource from "@/components/swarm/remove";
import ResourceUpdates from "@/components/updates/resource";
import { swarmNodeStateIntention } from "@/lib/color";
import { usePermissions, useRead, useSetTitle } from "@/lib/hooks";
import ResourceDescription from "@/resources/description";
import ResourceLink from "@/resources/link";
import { useSwarm } from "@/resources/swarm";
import { ICONS } from "@/theme/icons";
import DividedChildren from "@/ui/divided-children";
import EntityHeader from "@/ui/entity-header";
import EntityPage from "@/ui/entity-page";
import Section from "@/ui/section";
import { Center, Group, Loader, Stack, Text } from "@mantine/core";
import { useParams } from "react-router-dom";
import SwarmNodeTabs from "./tabs";

export default function SwarmNode() {
  const { id: swarmId, node: __node } = useParams() as {
    id: string;
    node: string;
  };
  const _node = decodeURIComponent(__node);
  const swarm = useSwarm(swarmId);
  const {
    data: node,
    isPending,
    isError,
  } = useRead("InspectSwarmNode", {
    swarm: swarmId,
    node: _node,
  });
  const state = node?.Status?.State;
  const { canExecute } = usePermissions({
    type: "Swarm",
    id: swarmId,
  });
  useSetTitle(
    `${swarm?.name} | Node | ${node?.Description?.Hostname ?? "Unknown"}`,
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
        <Text>Failed to inspect swarm node.</Text>
      </Center>
    );
  }

  if (!node) {
    return (
      <Center h="30vh">
        <Text>No swarm node found with given id: {_node}</Text>
      </Center>
    );
  }

  const intent = swarmNodeStateIntention(state);

  const Header = (
    <Stack justify="space-between">
      <Stack gap="md" pb="md" className="bordered-light" bdrs="md">
        <EntityHeader
          name={node?.Description?.Hostname}
          icon={ICONS.SwarmNode}
          intent={intent}
          state={state}
          status={node.Spec?.Role}
        />
        <DividedChildren px="md">
          <Text>Swarm Node</Text>
          <ResourceLink type="Swarm" id={swarmId} />
        </DividedChildren>
      </Stack>
      <ResourceDescription type="Swarm" id={swarmId} />
    </Stack>
  );

  return (
    <EntityPage backTo={"/swarms/" + swarmId}>
      <Stack hiddenFrom="xl" w="100%">
        {Header}
        <ResourceUpdates type="Swarm" id={swarmId} />
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
        <ResourceUpdates type="Swarm" id={swarmId} />
      </Group>

      <Stack mt="lg" gap="xl">
        {canExecute && node.ID && (
          <Section
            title="Execute"
            icon={<ICONS.Execution size="1.3rem" />}
            my="xl"
          >
            <RemoveSwarmResource
              swarmId={swarmId}
              type="Node"
              resourceId={node.ID}
              resourceName={node.Description?.Hostname}
            />
          </Section>
        )}

        {swarm && (
          <SwarmNodeTabs
            swarm={swarm}
            _node={_node}
            node={node}
            intent={intent}
          />
        )}
      </Stack>
    </EntityPage>
  );
}
