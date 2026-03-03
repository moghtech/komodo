import { ColorIntention, hexColorByIntention } from "@/lib/color";
import { ICONS } from "@/theme/icons";
import {
  ActionIcon,
  Group,
  GroupProps,
  HoverCard,
  Progress,
  ProgressProps,
  Text,
  TextProps,
} from "@mantine/core";
import { ReactNode } from "react";

export interface StatCellProps extends GroupProps {
  value: number | undefined;
  intent: ColorIntention;
  textProps?: TextProps;
  barProps?: ProgressProps;
  info?: ReactNode;
  infoDisabled?: boolean;
}

export default function StatCell({
  value,
  intent,
  textProps,
  barProps,
  info,
  infoDisabled,
  ...groupProps
}: StatCellProps) {
  const ProgressComponent = (
    <Progress
      value={value ?? 0}
      color={hexColorByIntention(intent)}
      w={200}
      size="xl"
      {...barProps}
    />
  );
  return (
    <Group
      gap="xs"
      justify="space-between"
      wrap="nowrap"
      {...groupProps}
    >
      <Text
        w={64}
        c={value === undefined ? "dimmed" : undefined}
        {...textProps}
      >
        {value === undefined ? "N/A" : value.toFixed(1) + "%"}
      </Text>
      {!info && ProgressComponent}
      {info && (
        <Group gap="xs" wrap="nowrap">
          {ProgressComponent}
          <HoverCard position="bottom-end">
            <HoverCard.Target>
              <ActionIcon variant="subtle" disabled={infoDisabled}>
                <ICONS.Info size="1rem" />
              </ActionIcon>
            </HoverCard.Target>
            <HoverCard.Dropdown>{info}</HoverCard.Dropdown>
          </HoverCard>
        </Group>
      )}
    </Group>
  );
}
