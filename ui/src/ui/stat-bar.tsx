import { hexColorByIntention } from "mogh_ui";
import { Group, Progress, StackProps, Text } from "@mantine/core";
import { ReactNode } from "react";
import InfoCard from "./info-card";

export interface StatBarProps extends StackProps {
  title: string;
  icon: ReactNode;
  description?: ReactNode;
  percentage: number | undefined;
  warning: number | undefined;
  critical: number | undefined;
}

export default function StatBar({
  title,
  icon,
  description,
  percentage: _percentage,
  warning: _warning,
  critical: _critical,
  ...props
}: StatBarProps) {
  const percentage = _percentage ?? 0;
  const warning = _warning ?? 100;
  const critical = _critical ?? 100;
  const intent =
    percentage > critical
      ? "Critical"
      : percentage > warning
        ? "Warning"
        : "Good";
  const color = hexColorByIntention(intent);
  return (
    <InfoCard
      title={title}
      info={
        <Group gap="xs">
          <Text c={color} fz="lg">
            {percentage.toFixed(2)}%
          </Text>
          {icon}
        </Group>
      }
      w={{ base: "100%", lg: 300 }}
      gap="0.2rem"
      justify="space-between"
      {...props}
    >
      {description && (
        <Text c="dimmed" size="sm">
          {description}
        </Text>
      )}
      <Progress color="bw" value={percentage} size="xl" />
    </InfoCard>
  );
}
