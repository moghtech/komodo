import RemoveSwarmResource from "@/components/swarm/remove";
import ResourceUpdates from "@/components/updates/resource";
import { usePermissions, useRead, useSetTitle } from "@/lib/hooks";
import ResourceDescription from "@/resources/description";
import ResourceLink from "@/resources/link";
import { useSwarm } from "@/resources/swarm";
import { ICONS } from "@/theme/icons";
import DividedChildren from "@/ui/divided-children";
import EntityHeader from "@/ui/entity-header";
import EntityPage from "@/ui/entity-page";
import Section from "@/ui/section";
import { Badge, Center, Group, Loader, Stack, Text } from "@mantine/core";
import { useParams } from "react-router-dom";
import SwarmConfigTabs from "./tabs";

export default function SwarmConfig() {
  const { id: swarmId, config: __config } = useParams() as {
    id: string;
    config: string;
  };
  const _config = decodeURIComponent(__config);
  const swarm = useSwarm(swarmId);
  const inUse = useRead("ListSwarmConfigs", { swarm: swarmId }).data?.find(
    (config) => config.ID === _config || config.Name === _config,
  )?.InUse;
  const {
    data: config,
    isPending,
    isError,
  } = useRead("InspectSwarmConfig", {
    swarm: swarmId,
    config: _config,
  });
  const { canExecute } = usePermissions({
    type: "Swarm",
    id: swarmId,
  });
  useSetTitle(`${swarm?.name} | Config | ${config?.Spec?.Name ?? "Unknown"}`);

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
        <Text>Failed to inspect swarm config.</Text>
      </Center>
    );
  }

  if (!config) {
    return (
      <Center h="30vh">
        <Text>No swarm config found with given id: {_config}</Text>
      </Center>
    );
  }

  const intent = inUse ? "Good" : "Critical";

  const Header = (
    <Stack justify="space-between">
      <Stack gap="md" pb="md" className="bordered-light" bdrs="md">
        <EntityHeader
          name={config?.Spec?.Name}
          icon={ICONS.SwarmConfig}
          intent={intent}
          state={
            !inUse && (
              <Badge variant="filled" color="red">
                Unused
              </Badge>
            )
          }
        />
        <DividedChildren px="md">
          <Text>Swarm Config</Text>
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
        {canExecute && config.ID && (
          <Section
            title="Execute"
            icon={<ICONS.Execution size="1.3rem" />}
            my="xl"
          >
            <RemoveSwarmResource
              swarmId={swarmId}
              type="Node"
              resourceId={config.ID}
              resourceName={config.Spec?.Name}
              disabled={inUse}
            />
          </Section>
        )}

        {swarm && (
          <SwarmConfigTabs swarm={swarm} config={_config} intent={intent} />
        )}
      </Stack>
    </EntityPage>
  );
}
