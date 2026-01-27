import { Badge, Group, Switch, SwitchProps } from "@mantine/core";

export interface EnableSwitchProps extends SwitchProps {
  checked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  redDisabled?: boolean;
}

export default function EnableSwitch({
  checked,
  color,
  label,
  onChange,
  onCheckedChange,
  disabled,
  redDisabled = true,
  ...props
}: EnableSwitchProps) {
  return (
    <Switch
      {...props}
      disabled={disabled}
      checked={checked}
      color={color}
      label={
        <Group>
          {label}
          <Badge
            color={checked ? color : redDisabled ? "red" : "gray"}
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
    />
  );
}
