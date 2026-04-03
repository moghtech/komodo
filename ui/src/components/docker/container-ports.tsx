import { useContainerPortsMap } from "@/lib/hooks";
import {
  Box,
  Group,
  GroupProps,
  HoverCard,
  HoverCardProps,
  Stack,
  Text,
} from "@mantine/core";
import { Types } from "komodo_client";
import { EthernetPort } from "lucide-react";
import { colorByIntention } from "@/lib/color";
import { ICONS } from "@/lib/icons";
import { fmtPortMount } from "@/lib/formatting";
import { useServerAddress } from "@/resources/server/hooks";
import DividedChildren from "@/ui/divided-children";

export interface ContainerPortsProps extends GroupProps {
  ports: Types.Port[];
  serverId: string | undefined;
  hoverCardProps?: HoverCardProps;
}

export default function ContainerPorts({
  ports,
  serverId,
  hoverCardProps,
  gap = "xs",
  wrap = "nowrap",
  ...groupProps
}: ContainerPortsProps) {
  const portsMap = useContainerPortsMap(ports);

  if (Object.keys(portsMap).length > 0) {
    console.log("mapped", portsMap);
  }

  const sortedNumericPorts = Object.keys(portsMap)
    .map(Number)
    .filter((port) => !Number.isNaN(port))
    .sort((a, b) => a - b);

  if (Object.keys(portsMap).length > 0) {
    console.log("sorted", sortedNumericPorts);
  }

  type Group = { start: number; end: number; ports: Types.Port[] };

  const groupedPorts = sortedNumericPorts.reduce<Group[]>((acc, port) => {
    const lastGroup = acc[acc.length - 1];
    const currentPorts = portsMap[String(port)] || [];
    if (lastGroup && port === lastGroup.end + 1) {
      lastGroup.end = port;
      lastGroup.ports.push(...currentPorts);
    } else {
      acc.push({ start: port, end: port, ports: currentPorts });
    }
    return acc;
  }, []);

  if (!groupedPorts.length) {
    return null;
  }

  console.log("grouped", groupedPorts);

  return (
    <DividedChildren gap={gap} wrap="nowrap" {...groupProps}>
      {groupedPorts.map((group) => (
        <Box key={group.start}>
          <ContainerPort
            ports={group.ports}
            hostPort={String(group.start)}
            serverId={serverId}
            {...hoverCardProps}
          />
        </Box>
      ))}
    </DividedChildren>
  );
}

export interface ContainerPortProps extends HoverCardProps {
  ports: Types.Port[];
  hostPort: string;
  serverId: string | undefined;
}

export function ContainerPort({
  ports,
  hostPort,
  serverId,
  position = "bottom",
  ...hoverCardProps
}: ContainerPortProps) {
  const serverAddress = useServerAddress(serverId);

  const isHttps = serverAddress?.protocol === "https:";
  const link =
    serverAddress &&
    (hostPort === "443" && isHttps
      ? `https://${serverAddress.hostname}`
      : `http://${serverAddress.hostname}:${hostPort}`);

  const uniqueHostPorts = Array.from(
    new Set(
      ports
        .map((p) => p.PublicPort)
        .filter((p): p is number => typeof p === "number")
        .map((n) => Number(n))
        .filter((n) => !Number.isNaN(n)),
    ),
  ).sort((a, b) => a - b);
  const displayText =
    uniqueHostPorts.length <= 1
      ? String(uniqueHostPorts[0] ?? hostPort)
      : `${uniqueHostPorts[0]}-${uniqueHostPorts[uniqueHostPorts.length - 1]}`;

  return (
    <HoverCard position={position} {...hoverCardProps}>
      <HoverCard.Target>
        <Group
          renderRoot={(props) =>
            link ? (
              <a target="_blank" href={link} {...props} />
            ) : (
              <div {...props} />
            )
          }
          gap="sm"
          wrap="nowrap"
        >
          <EthernetPort size="1rem" color={colorByIntention("Good")} />
          {displayText}
        </Group>
      </HoverCard.Target>
      <HoverCard.Dropdown>
        <Stack gap="xs">
          <Group
            renderRoot={(props) =>
              link ? (
                <a target="_blank" href={link} {...props} />
              ) : (
                <div {...props} />
              )
            }
            wrap="nowrap"
          >
            {link ? (
              <>
                <ICONS.Link size="1rem" color={colorByIntention("Good")} />
                {link}
              </>
            ) : (
              "Missing external address"
            )}
          </Group>

          {ports.slice(0, 10).map((port, i) => (
            <Group key={i} c="dimmed">
              <Text>-</Text>
              <Text>{fmtPortMount(port)}</Text>
            </Group>
          ))}

          {ports.length > 10 && (
            <Group c="dimmed">
              <Text>+</Text>
              <Text>{ports.length - 10} more...</Text>
            </Group>
          )}
        </Stack>
      </HoverCard.Dropdown>
    </HoverCard>
  );
}
