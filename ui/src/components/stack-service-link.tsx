import {
  containerStateIntention,
  hexColorByIntention,
  swarmStateIntention,
} from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { Group, Text } from "@mantine/core";
import { Link } from "react-router-dom";

export interface StackServiceLinkProps {
  id: string;
  service: string;
}

export default function StackServiceLink({
  id,
  service: _service,
}: StackServiceLinkProps) {
  const services = useRead(
    "ListStackServices",
    { stack: id },
    { refetchInterval: 10_000 },
  ).data;
  const service = services?.find((s) => s.service === _service);
  const intention = service?.swarm_service?.State
    ? swarmStateIntention(service?.swarm_service?.State)
    : containerStateIntention(service?.container?.state);
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
