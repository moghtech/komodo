import {
  colorByIntention,
  ColorIntention,
  hexColorByIntention,
} from "@/lib/color";
import { Group, Stack, Text } from "@mantine/core";
import { ReactNode } from "react";

export default function EntityHeader({
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
    <Group
      gap="md"
      px="xl"
      py="md"
      style={{
        background,
        borderTopLeftRadius: "var(--mantine-radius-md)",
        borderTopRightRadius: "var(--mantine-radius-md)",
      }}
    >
      {icon}
      <Stack gap="0">
        {name && (
          <Text fz="h1" fw="600">
            {name}
          </Text>
        )}
        <Group fz="md" tt="uppercase" mt="-8" gap="sm">
          <Text c={color} fw="600">
            {state}
          </Text>
          <Text c="dimmed">{status}</Text>
        </Group>
      </Stack>
    </Group>
  );
}
