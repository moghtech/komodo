import { ColorIntention, hexColorByIntention } from "mogh_ui";
import { ICONS } from "@/lib/icons";
import {
  ActionIcon,
  FloatingPosition,
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
  infoPosition?: FloatingPosition;
  infoDisabled?: boolean;
  suffix?: string;
}

export default function StatCell({
  value,
  intent,
  textProps,
  barProps,
  info,
  infoPosition = "left-start",
  infoDisabled,
  suffix = "%",
  ...groupProps
}: StatCellProps) {
  const ProgressComponent = (
    <Progress
      value={value ?? 0}
      color={hexColorByIntention(intent)}
      flex={1}
      h={15}
      radius="xs"
      {...barProps}
    />
  );
  return (
    <Group gap={4} justify="flex-start" wrap="nowrap" w="100%" {...groupProps}>
      <Text
        w={42}
        size="md"
        c={value === undefined ? "dimmed" : undefined}
        {...textProps}
      >
        {value === undefined ? "N/A" : value.toFixed(0) + suffix}
      </Text>
      {!info && ProgressComponent}
      {info && (
        <HoverCard position={infoPosition} disabled={infoDisabled}>
          <HoverCard.Target>
            <Group gap={2} wrap="nowrap" flex={1}>
              {ProgressComponent}
              <ActionIcon variant="subtle" size="sm" disabled={infoDisabled}>
                <ICONS.Info size="1rem" />
              </ActionIcon>
            </Group>
          </HoverCard.Target>
          <HoverCard.Dropdown>{info}</HoverCard.Dropdown>
        </HoverCard>
      )}
    </Group>
  );
}
