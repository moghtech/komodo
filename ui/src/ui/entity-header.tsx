import { ColorIntention, hexColorByIntention } from "@/lib/color";
import { Group, Stack, Text } from "@mantine/core";
import { FC, ReactNode } from "react";

export interface EntityHeaderProps {
  name?: string;
  icon: FC<{ size?: string | number; color?: string }>;
  intent: ColorIntention;
  state?: ReactNode;
  status?: ReactNode;
  action?: ReactNode;
}

export default function EntityHeader({
  name,
  icon: Icon,
  intent,
  state,
  status,
  action,
}: EntityHeaderProps) {
  const color = hexColorByIntention(intent);
  const background = color ? color + "25" : undefined;
  return (
    <Group
      justify="space-between"
      gap="lg"
      px="xl"
      py="md"
      style={{
        background,
        borderTopLeftRadius: "var(--mantine-radius-md)",
        borderTopRightRadius: "var(--mantine-radius-md)",
      }}
    >
      <Group gap="lg">
        <Icon size="2rem" color={color} />
        <Stack gap="0">
          {name && (
            <Text fz="h1" fw="bolder">
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
      {action}
    </Group>
  );
}
