import { containerStateIntention, swarmStateIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { useStack } from "@/resources/stack";
import { Group, Text } from "@mantine/core";
import { Types } from "komodo_client";
import { ColorIntention, hexColorByIntention } from "mogh_ui";
import { Link } from "react-router-dom";

export interface StackServiceLinkProps {
  id: string;
  service: string;
}

export default function StackServiceLink({
  id,
  service: _service,
}: StackServiceLinkProps) {
  const isUnknown = useStack(id)?.info.state === Types.StackState.Unknown;
  const services = useRead(
    "ListStackServices",
    { stack: id },
    { refetchInterval: 10_000 },
  ).data;
  const service = services?.find((s) => s.service === _service);
  const intention: ColorIntention = service?.swarm_service?.State
    ? swarmStateIntention(service?.swarm_service?.State)
    : service?.container?.state
      ? containerStateIntention(service?.container?.state)
      : isUnknown
        ? "Unknown"
        : "Neutral";
  const color = hexColorByIntention(intention);
  return (
    <Group
      renderRoot={(props) => (
        <Link to={`/stacks/${id}/service/${_service}`} {...props} />
      )}
      onClick={(e) => e.stopPropagation()}
      wrap="nowrap"
      gap="xs"
    >
      <ICONS.Service size="1rem" color={color} />
      <Text className="hover-underline" style={{ textWrap: "nowrap" }}>
        {_service}
      </Text>
    </Group>
  );
}
