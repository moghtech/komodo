import { useRead } from "@/lib/hooks";
import { useServer } from ".";
import { Group, HoverCard, Text } from "@mantine/core";
import { ICONS } from "@/lib/icons";
import { Types } from "komodo_client";
import { hexColorByIntention } from "mogh_ui";

export default function ServerVersion({ id }: { id: string }) {
  const coreVersion = useRead("GetVersion", {}).data?.version;
  const server = useServer(id);
  const serverVersion = server?.info.version;
  const mismatch =
    !!coreVersion && !!serverVersion && coreVersion !== serverVersion;

  return (
    <HoverCard width={300} position="bottom-start">
      <HoverCard.Target>
        <Group gap="xs" wrap="nowrap">
          {!serverVersion ? (
            <ICONS.Alert size="1rem" color={hexColorByIntention("Unknown")} />
          ) : mismatch ? (
            <ICONS.Alert size="1rem" color={hexColorByIntention("Critical")} />
          ) : (
            <ICONS.Check size="1rem" color={hexColorByIntention("Good")} />
          )}
          <Text>{serverVersion ?? "Unknown"}</Text>
        </Group>
      </HoverCard.Target>
      <HoverCard.Dropdown>
        {server?.info.state === Types.ServerState.Disabled ? (
          <Text>
            Server version is <b>disabled</b>.
          </Text>
        ) : !serverVersion ? (
          <Text>
            Periphery version is <b>unknown</b>.
          </Text>
        ) : mismatch ? (
          <Text>
            Periphery version <b>mismatch</b>. Expected <b>{coreVersion}</b>.
          </Text>
        ) : (
          <Text>
            Periphery and Core version <b>match</b>.
          </Text>
        )}
      </HoverCard.Dropdown>
    </HoverCard>
  );
}
