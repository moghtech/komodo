import { ICONS } from "@/lib/icons";
import { Button, Code, Group, HoverCard, Stack, Text } from "@mantine/core";

export interface HoverErrorProps {
  error: string;
  trace: string[];
}

export default function HoverError({ error, trace }: HoverErrorProps) {
  return (
    <HoverCard position="bottom-start">
      <HoverCard.Target>
        <Button
          variant="filled"
          color="red"
          leftSection={<ICONS.Alert size="1rem" />}
        >
          Error
        </Button>
      </HoverCard.Target>
      <HoverCard.Dropdown>
        <Stack gap="sm" maw={{ base: 400, xs: 500, sm: 600, md: 800 }}>
          <Stack component={Code} gap="0">
            <Text fw="bold" c="red" fz="sm">
              ERROR:
            </Text>
            <Text fz="sm">{error}</Text>
          </Stack>
          {trace.length > 0 && (
            <Stack component={Code} gap="0">
              <Text c="dimmed" fz="sm">
                TRACE:
              </Text>
              {trace.map((error, i) => (
                <Group key={i} wrap="nowrap" gap="xs" align="start" pl="1rem">
                  <Text c="dimmed" fz="sm">
                    {i + 1}:
                  </Text>
                  <Text fz="sm">{error}</Text>
                </Group>
              ))}
            </Stack>
          )}
        </Stack>
      </HoverCard.Dropdown>
    </HoverCard>
  );
}
