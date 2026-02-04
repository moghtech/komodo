import { Fragment } from "react";
import { useContainerPortsMap, useServerAddress } from "@/lib/hooks";
import { Group, Stack, Text, Tooltip } from "@mantine/core";
import { Types } from "komodo_client";
import { EthernetPort } from "lucide-react";
import { colorByIntention } from "@/lib/color";
import { ICONS } from "@/theme/icons";
import { fmtPortMount } from "@/lib/formatting";

export interface ContainerPortsProps {
  ports: Types.Port[];
  serverId: string | undefined;
}

export function ContainerPorts({ ports, serverId }: ContainerPortsProps) {
  const portsMap = useContainerPortsMap(ports);
  const sortedNumericPorts = Object.keys(portsMap)
    .map(Number)
    .filter((port) => !Number.isNaN(port))
    .sort((a, b) => a - b);

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

  return (
    <Group>
      {groupedPorts.map((group, i) => (
        <Fragment key={group.start}>
          {i > 0 && (
            <Text component="span" c="dimmed">
              |
            </Text>
          )}
          <ContainerPort
            ports={group.ports}
            hostPort={String(group.start)}
            serverId={serverId}
          />
        </Fragment>
      ))}
    </Group>
  );
}

export interface ContainerPortProps {
  ports: Types.Port[];
  hostPort: string;
  serverId: string | undefined;
}

export function ContainerPort({
  ports,
  hostPort,
  serverId,
}: ContainerPortProps) {
  const serverAddress = useServerAddress(serverId);

  if (!serverAddress) return null;

  const isHttps = serverAddress.protocol === "https:";
  const link =
    hostPort === "443" && isHttps
      ? `https://${serverAddress.hostname}`
      : `http://${serverAddress.hostname}:${hostPort}`;

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
    <Tooltip
      label={
        <Stack>
          <Group
            renderRoot={(props) => <a target="_blank" href={link} {...props} />}
          >
            <ICONS.Link size="1rem" color={colorByIntention("Good")} />
            {link}
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
      }
    >
      <Group
        renderRoot={(props) => <a target="_blank" href={link} {...props} />}
      >
        <EthernetPort size="1rem" color={colorByIntention("Good")} />
        {displayText}
      </Group>
    </Tooltip>
  );
}
