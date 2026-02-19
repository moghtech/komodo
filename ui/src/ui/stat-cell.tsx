import { ColorIntention, hexColorByIntention } from "@/lib/color";
import {
  Group,
  GroupProps,
  Progress,
  ProgressProps,
  Text,
  TextProps,
} from "@mantine/core";

export interface StatCellProps extends GroupProps {
  value: number;
  intent: ColorIntention;
  textProps?: TextProps;
  barProps?: ProgressProps;
}

export default function StatCell({
  value,
  intent,
  textProps,
  barProps,
  ...groupProps
}: StatCellProps) {
  return (
    <Group gap="xs" justify="space-between" wrap="nowrap" {...groupProps}>
      <Text {...textProps}>{value.toFixed(1)}%</Text>
      <Progress
        value={value}
        color={hexColorByIntention(intent)}
        w="70%"
        miw={80}
        size="xl"
        {...barProps}
      />
    </Group>
  );
}
