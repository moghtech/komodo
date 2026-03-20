import { useRead } from "@/lib/hooks";
import StatusBadge from "@/ui/status-badge";
import { Box, Group, HoverCard, Loader, Stack, Text } from "@mantine/core";
import SwarmResourceLink, { SwarmResourceType } from "@/components/swarm/link";
import { ColorIntention } from "@/lib/color";

export interface SwarmHeaderInfoProps<R> {
  swarmId: string;
  type: SwarmResourceType;
  resourceId: (resource: R) => string | undefined;
  resourceName: (resource: R) => string | undefined;
  resourceState: (resource: R) => string | undefined;
  resourceStateIntent: (resource: R) => ColorIntention;
}

export default function SwarmHeaderInfo<R>({
  swarmId,
  type,
  resourceId,
  resourceName,
  resourceState,
  resourceStateIntent,
}: SwarmHeaderInfoProps<R>) {
  const resources = useRead(`ListSwarm${type}s`, { swarm: swarmId }).data;
  return (
    <Box>
      <HoverCard position="bottom-start">
        <HoverCard.Target>
          <Text>
            {resources ? (
              <>
                <b>{resources.length}</b>{" "}
                {`${type.toLowerCase()}${resources.length === 1 ? "" : "s"}`}
              </>
            ) : (
              <Loader size="xs" />
            )}
          </Text>
        </HoverCard.Target>
        <HoverCard.Dropdown>
          <Stack gap="xs">
            {resources?.map((resource) => (
              <Group
                justify="space-between"
                className="bordered-light"
                p="sm"
                bdrs="sm"
              >
                <SwarmResourceLink
                  swarmId={swarmId}
                  type={type}
                  resourceId={resourceId(resource as R)}
                  name={resourceName(resource as R)}
                />
                <StatusBadge
                  text={resourceState(resource as R)}
                  intent={resourceStateIntent(resource as R)}
                />
              </Group>
            ))}
          </Stack>
        </HoverCard.Dropdown>
      </HoverCard>
    </Box>
  );
}
