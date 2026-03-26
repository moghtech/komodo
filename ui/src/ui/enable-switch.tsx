import { Badge, Group, GroupProps, Switch, SwitchProps } from "@mantine/core";

export interface EnableSwitchProps extends SwitchProps {
  checked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  labelProps?: GroupProps;
}

export default function EnableSwitch({
  checked,
  color = "green.9",
  label,
  onChange,
  onCheckedChange,
  disabled,
  labelProps,
  ...props
}: EnableSwitchProps) {
  return (
    <Switch
      disabled={disabled}
      checked={checked}
      color={color}
      label={
        <Group gap="sm" wrap="nowrap" {...labelProps}>
          {label}
          <Badge
            color={checked ? color : "gray"}
            opacity={disabled ? 0.7 : 1}
            style={{ cursor: disabled ? undefined : "pointer" }}
          >
            {checked ? "Enabled" : "Disabled"}
          </Badge>
        </Group>
      }
      onChange={(e) => {
        onChange?.(e);
        onCheckedChange?.(e.target.checked);
      }}
      {...props}
    />
  );
}
