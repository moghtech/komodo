import { useWebsocketConnected, useWebsocketReconnect } from "@/lib/socket";
import { ActionIcon, Box, HoverCard, Text } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { Circle } from "lucide-react";
import { hexColorByIntention } from "mogh_ui";

export default function WebsocketStatus() {
  const connected = useWebsocketConnected();
  const reconnect = useWebsocketReconnect();
  const onClick = () => {
    reconnect();
    notifications.show({
      message: connected
        ? "Triggered websocket reconnect"
        : "Triggered websocket connect",
      color: "blue",
    });
  };
  const intention = connected ? "Good" : "Critical";
  const color = hexColorByIntention(intention);

  const Target = (
    <ActionIcon variant="subtle" onClick={onClick} size="xl">
      <Circle
        size="1.2rem"
        color={color}
        fill={color}
        style={{ transition: "all 300ms ease" }}
      />
    </ActionIcon>
  );

  return (
    <>
      {/* The hovercard can open unexpectedly on mobile so is hidden */}
      <Box hiddenFrom="sm">{Target}</Box>
      <Box visibleFrom="sm">
        <HoverCard offset={16}>
          <HoverCard.Target>{Target}</HoverCard.Target>
          <HoverCard.Dropdown>
            <Text>Websocket Status</Text>
            <Text c="dimmed" fz="sm">
              Click to reconnect
            </Text>
          </HoverCard.Dropdown>
        </HoverCard>
      </Box>
    </>
  );
}
