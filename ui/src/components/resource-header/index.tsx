import {
  colorByIntention,
  ColorIntention,
  hexColorByIntention,
} from "@/lib/color";
import { Flex, Group, Stack, Text } from "@mantine/core";
import { ReactNode } from "react";

export default function ResourceHeader({
  name,
  icon,
  intent,
  state,
  status,
}: {
  name?: string;
  icon: ReactNode;
  intent: ColorIntention;
  state?: ReactNode;
  status?: ReactNode;
}) {
  const color = colorByIntention(intent);
  const _background = hexColorByIntention(intent);
  const background = _background ? _background + "25" : undefined;

  return (
    <Flex
      align="center"
      justify="space-between"
      gap="md"
      px="md"
      py="xs"
      style={{
        background,
        borderTopLeftRadius: "var(--mantine-radius-md)",
        borderTopRightRadius: "var(--mantine-radius-md)",
      }}
    >
      <Group>
        {icon}
        <Stack gap="0">
          {name && (
            <Text fz="h1" fw="600">
              {name}
            </Text>
          )}
          <Group fz="md" tt="uppercase" mt="-8">
            <Text c={color} fw="600">
              {state}
            </Text>
            <Text c="dimmed">{status}</Text>
          </Group>
        </Stack>
      </Group>
    </Flex>
  );
}
