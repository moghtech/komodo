import { hexColorByIntention } from "@/lib/color";
import { useWebsocketConnected, useWebsocketReconnect } from "@/lib/socket";
import { ActionIcon, HoverCard, Text } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { Circle } from "lucide-react";

export default function WebsocketStatus() {
  const connected = useWebsocketConnected();
  const reconnect = useWebsocketReconnect();
  const onClick = () => {
    reconnect();
    notifications.show({
      message: connected
        ? "Triggered websocket reconnect"
        : "Triggered websocket connect",
    });
  };
  const intention = connected ? "Good" : "Critical";
  const color = hexColorByIntention(intention);

  return (
    <HoverCard position="bottom-end" offset={20}>
      <HoverCard.Target>
        <ActionIcon variant="subtle" onClick={onClick} size="xl">
          <Circle
            size="1.2rem"
            color={color}
            fill={color}
            style={{ transition: "all 300ms ease" }}
          />
        </ActionIcon>
      </HoverCard.Target>
      <HoverCard.Dropdown>
        <Text>Websocket Status</Text>
        <Text c="dimmed" fz="sm">
          Click to reconnect
        </Text>
      </HoverCard.Dropdown>
    </HoverCard>
  );
}
