import {
  Badge,
  Box,
  Code,
  Group,
  HoverCard,
  HoverCardProps,
  Stack,
  Text,
} from "@mantine/core";

export interface HashCompareProps extends HoverCardProps {
  /* The last deployed / built / synced hash */
  lastHash: string | undefined;
  lastMessage: string | undefined;
  lastLabel: string | undefined;
  /* The latest hash from the repo source */
  latestHash: string | undefined;
  latestMessage: string | undefined;
}

export default function HashCompare({
  lastHash,
  lastMessage,
  lastLabel = "last",
  latestHash,
  latestMessage,
  ...props
}: HashCompareProps) {
  const outOfDate =
    !!lastHash && lastHash.slice(0, 8) !== latestHash?.slice(0, 8);
  return (
    <HoverCard position="bottom-start" {...props}>
      <HoverCard.Target>
        <Box
          px="sm"
          py="0.2rem"
          bdrs="sm"
          style={{
            borderColor: outOfDate
              ? "var(--mantine-color-yellow-7)"
              : "var(--mantine-color-accent-border-1)",
            borderStyle: "solid",
            borderWidth: "1px",
            cursor: "pointer",
          }}
        >
          {lastHash ? lastLabel : "latest"}: {lastHash || latestHash}
        </Box>
      </HoverCard.Target>
      <HoverCard.Dropdown>
        <Stack>
          <Stack gap="xs">
            <Group gap="xs">
              <Badge color="accent.9">message</Badge>
              <Text c="dimmed">{lastHash}</Text>
            </Group>
            <Code>{lastMessage || latestMessage}</Code>
          </Stack>
          {outOfDate && (
            <Stack gap="xs">
              <Group gap="xs">
                <Badge style={{ borderColor: "var(--mantine-color-yellow-7)" }}>
                  latest
                </Badge>
                <Text c="dimmed">{latestHash}</Text>
              </Group>
              <Code>{latestMessage}</Code>
            </Stack>
          )}
        </Stack>
      </HoverCard.Dropdown>
    </HoverCard>
  );
}
