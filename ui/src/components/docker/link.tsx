import { ReactNode } from "react";
import { Types } from "komodo_client";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { hexColorByIntention, containerStateIntention } from "@/lib/color";
import { Box, Group, Text } from "@mantine/core";
import { Link } from "react-router-dom";

export type DockerResourceType = "Container" | "Network" | "Image" | "Volume";

export interface DockerResourceLinkProps {
  type: DockerResourceType;
  serverId: string;
  name: string | undefined;
  id?: string;
  extra?: ReactNode;
  dimmed?: boolean;
}

export default function DockerResourceLink({
  type,
  serverId,
  name,
  id,
  extra,
  dimmed,
}: DockerResourceLinkProps) {
  if (!name) return "Unknown";

  const Icon = DOCKER_LINK_ICONS[type];

  return (
    <Group
      renderRoot={(props) => (
        <Link
          to={`/servers/${serverId}/${type.toLowerCase()}/${encodeURIComponent(name)}`}
          {...props}
        />
      )}
      className="hover-underline"
      c={dimmed ? "dimmed" : undefined}
      wrap="nowrap"
      gap="xs"
    >
      <Icon serverId={serverId} name={type === "Image" ? id : name} />
      <Text className="text-ellipsis" maw={{ base: 250, lg: 300 }} title={name}>
        {name}
      </Text>
      {extra && <Box style={{ textDecorationLine: "none" }}>{extra}</Box>}
    </Group>
  );
}

export const DOCKER_LINK_ICONS: {
  [type in DockerResourceType]: React.FC<{
    serverId: string;
    name: string | undefined;
    size?: string | number;
  }>;
} = {
  Container: ({ serverId, name, size = "1rem" }) => {
    const state =
      useRead("ListDockerContainers", { server: serverId }).data?.find(
        (container) => container.name === name,
      )?.state ?? Types.ContainerStateStatusEnum.Empty;
    return (
      <ICONS.Container
        size={size}
        color={hexColorByIntention(containerStateIntention(state))}
      />
    );
  },
  Network: ({ serverId, name, size = "1rem" }) => {
    const containers =
      useRead("ListDockerContainers", { server: serverId }).data ?? [];
    const noContainers = !name
      ? false
      : containers.every((container) => !container.networks?.includes(name));
    const intention = !name
      ? "Warning"
      : noContainers
        ? ["none", "host", "bridge"].includes(name)
          ? "None"
          : "Critical"
        : "Good";
    return <ICONS.Network size={size} color={hexColorByIntention(intention)} />;
  },
  Image: ({ serverId, name, size = "1rem" }) => {
    const containers =
      useRead("ListDockerContainers", { server: serverId }).data ?? [];
    const noContainers = !name
      ? false
      : containers.every((container) => container.image_id !== name);
    const intention = !name ? "Warning" : noContainers ? "Critical" : "Good";
    return <ICONS.Image size={size} color={hexColorByIntention(intention)} />;
  },
  Volume: ({ serverId, name, size = "1rem" }) => {
    const containers =
      useRead("ListDockerContainers", { server: serverId }).data ?? [];
    const noContainers = !name
      ? false
      : containers.every((container) => !container.volumes?.includes(name));
    const intention = !name ? "Warning" : noContainers ? "Critical" : "Good";
    return <ICONS.Volume size={size} color={hexColorByIntention(intention)} />;
  },
};
