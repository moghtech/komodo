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
}

export default function StatCell({
  value,
  intent,
  textProps,
  barProps,
  info,
  infoPosition = "left-start",
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
    <Group gap="xs" justify="space-between" wrap="nowrap" {...groupProps}>
      <Text
        w={64}
        c={value === undefined ? "dimmed" : undefined}
        {...textProps}
      >
        {value === undefined ? "N/A" : value.toFixed(1) + "%"}
      </Text>
      {!info && ProgressComponent}
      {info && (
        <HoverCard position={infoPosition} disabled={infoDisabled}>
          <HoverCard.Target>
            <Group gap="xs" wrap="nowrap">
              {ProgressComponent}
              <ActionIcon variant="subtle" disabled={infoDisabled}>
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
