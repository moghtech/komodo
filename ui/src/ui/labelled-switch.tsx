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
  labelProps?: TextProps;
}

export default function LabelledSwitch({
  checked,
  onCheckedChange,
  label,
  groupProps,
  labelProps,
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
      justify="space-between"
      w={{ base: "100%", xs: "fit-content" }}
      {...groupProps}
    >
      <Text c={checked ? undefined : "dimmed"} {...labelProps}>
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
