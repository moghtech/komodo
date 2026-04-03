import {
  hexColorByIntention,
  swarmNodeStateIntention,
  swarmStateIntention,
  swarmTaskStateIntention,
} from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { Box, Group, Text } from "@mantine/core";
import { ReactNode } from "react";
import { Link } from "react-router-dom";

export type SwarmResourceType =
  | "Node"
  | "Service"
  | "Task"
  | "Secret"
  | "Config"
  | "Stack";

export interface SwarmResourceLinkProps {
  type: SwarmResourceType;
  swarmId: string;
  resourceId: string | undefined;
  name: string | undefined;
  extra?: ReactNode;
  dimmed?: boolean;
}

export default function SwarmResourceLink({
  type,
  swarmId,
  resourceId,
  name,
  extra,
  dimmed,
}: SwarmResourceLinkProps) {
  if (!resourceId) return "Unknown";

  const Icon = SWARM_LINK_ICONS[type];

  return (
    <Group
      title={`Swarm ${type} - ${name}`}
      renderRoot={(props) => (
        <Link
          to={`/swarms/${swarmId}/swarm-${type.toLowerCase()}/${encodeURIComponent(resourceId)}`}
          {...props}
        />
      )}
      className="hover-underline"
      c={dimmed ? "dimmed" : undefined}
      wrap="nowrap"
      gap="xs"
    >
      <Icon swarmId={swarmId} resourceId={resourceId} />
      <Text className="text-ellipsis" maw={{ base: 250, lg: 300 }}>
        {name}
      </Text>
      {extra && <Box style={{ textDecorationLine: "none" }}>{extra}</Box>}
    </Group>
  );
}

export const SWARM_LINK_ICONS: {
  [type in SwarmResourceType]: React.FC<{
    swarmId?: string;
    resourceId?: string;
    size?: number;
  }>;
} = {
  Node: ({ swarmId, resourceId, size = "1rem" }) => {
    const state = useRead(
      "ListSwarmNodes",
      { swarm: swarmId! },
      { enabled: !!swarmId },
    ).data?.find((node) => resourceId && node.ID === resourceId)?.State;
    return (
      <ICONS.SwarmNode
        size={size}
        color={hexColorByIntention(swarmNodeStateIntention(state))}
      />
    );
  },
  Stack: ({ swarmId, resourceId, size = "1rem" }) => {
    const state = useRead(
      "ListSwarmStacks",
      { swarm: swarmId! },
      { enabled: !!swarmId },
    ).data?.find((stack) => resourceId && stack.Name === resourceId)?.State;
    return (
      <ICONS.SwarmStack
        size={size}
        color={hexColorByIntention(swarmStateIntention(state))}
      />
    );
  },
  Service: ({ swarmId, resourceId, size = "1rem" }) => {
    const state = useRead(
      "ListSwarmServices",
      { swarm: swarmId! },
      { enabled: !!swarmId },
    ).data?.find(
      (service) =>
        resourceId &&
        (service.ID === resourceId || service.Name === resourceId),
    )?.State;
    return (
      <ICONS.SwarmService
        size={size}
        color={hexColorByIntention(swarmStateIntention(state))}
      />
    );
  },
  Task: ({ swarmId, resourceId, size = "1rem" }) => {
    const task = useRead(
      "ListSwarmTasks",
      { swarm: swarmId! },
      { enabled: !!swarmId },
    ).data?.find((task) => resourceId && task.ID === resourceId);
    return (
      <ICONS.SwarmTask
        size={size}
        color={hexColorByIntention(
          swarmTaskStateIntention(task?.State, task?.DesiredState),
        )}
      />
    );
  },
  Config: ({ swarmId, resourceId, size = "1rem" }) => {
    const inUse = useRead(
      "ListSwarmConfigs",
      { swarm: swarmId! },
      { enabled: !!swarmId },
    ).data?.find(
      (config) =>
        resourceId && (config.ID === resourceId || config.Name === resourceId),
    )?.InUse;
    return (
      <ICONS.SwarmConfig
        size={size}
        color={hexColorByIntention(inUse ? "Good" : "Critical")}
      />
    );
  },
  Secret: ({ swarmId, resourceId, size = "1rem" }) => {
    const inUse = useRead(
      "ListSwarmSecrets",
      { swarm: swarmId! },
      { enabled: !!swarmId },
    ).data?.find(
      (secret) =>
        resourceId && (secret.ID === resourceId || secret.Name === resourceId),
    )?.InUse;
    return (
      <ICONS.SwarmSecret
        size={size}
        color={hexColorByIntention(inUse ? "Good" : "Critical")}
      />
    );
  },
};
