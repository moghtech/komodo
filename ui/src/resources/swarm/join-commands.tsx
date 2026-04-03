import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { CopyButton } from "mogh_ui";
import { Button, Center, Group, Loader, Stack, Text } from "@mantine/core";
import { Types } from "komodo_client";

export default function JoinSwarmCommands({
  id,
  close,
}: {
  id: string;
  close: () => void;
}) {
  const addr = useRead("ListSwarmNodes", { swarm: id }).data?.find(
    (node) => node.State === Types.NodeState.READY && node.ManagerAddr,
  )?.ManagerAddr;
  const tokens = useRead("InspectSwarm", { swarm: id }).data?.JoinTokens;
  const managerCmd = `docker swarm join --token ${tokens?.Manager} ${addr}`;
  const workerCmd = `docker swarm join --token ${tokens?.Worker} ${addr}`;

  return (
    <Stack>
      <Text c="dimmed">
        Copy a command below and run it on the target host to join the swarm.
      </Text>

      {addr && tokens ? (
        <>
          <Group justify="space-between" gap="sm">
            <Text>As Manager</Text>
            <Group gap="sm" wrap="nowrap">
              <Text
                ff="monospace"
                title={managerCmd}
                className="text-ellipsis accent-hover-light bordered-heavy"
                bdrs="md"
                px="md"
                py="0.4rem"
                w={600}
                maw="70vw"
              >
                {managerCmd}
              </Text>
              <CopyButton content={managerCmd} />
            </Group>
          </Group>

          <Group justify="space-between">
            <Text>As Worker</Text>
            <Group gap="sm" wrap="nowrap">
              <Text
                ff="monospace"
                title={workerCmd}
                className="text-ellipsis accent-hover-light bordered-heavy"
                bdrs="md"
                px="md"
                py="0.4rem"
                w={600}
                maw="70vw"
              >
                {workerCmd}
              </Text>
              <CopyButton content={workerCmd} />
            </Group>
          </Group>
        </>
      ) : (
        <Center>
          <Loader size="md" />
        </Center>
      )}

      <Group justify="end">
        <Button leftSection={<ICONS.Check size="1rem" />} onClick={close}>
          Close
        </Button>
      </Group>
    </Stack>
  );
}
