import {
  Group,
  GroupProps,
  Switch,
  SwitchProps,
  Text,
  TextProps,
} from "@mantine/core";
import { ReactNode } from "react";

export interface LabelledSwitchProps extends SwitchProps {
  checked: boolean | undefined;
  onCheckedChange: (checked: boolean) => void;
  label?: ReactNode;
  groupProps?: GroupProps;
  textProps?: TextProps;
}

export default function LabelledSwitchProps({
  checked,
  onCheckedChange,
  label,
  groupProps,
  textProps,
  ...switchProps
}: LabelledSwitchProps) {
  return (
    <Group
      gap="xs"
      onClick={(e) => {
        e.preventDefault();
        onCheckedChange(!checked);
      }}
      className="bordered-light"
      px="xs"
      py={6}
      bdrs="sm"
      style={{ cursor: "pointer" }}
      {...groupProps}
    >
      <Text c="dimmed" {...textProps}>
        {label}
      </Text>
      <Switch
        checked={checked}
        style={{ pointerEvents: "none" }}
        {...switchProps}
      />
    </Group>
  );
}
