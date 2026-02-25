import { fmtSnakeCaseToUpperSpaceCase } from "@/lib/formatting";
import { Select, SelectProps } from "@mantine/core";
import { Types } from "komodo_client";

export interface PermissionLevelSelectorProps extends Omit<
  SelectProps,
  "onChange"
> {
  level: Types.PermissionLevel;
  onChange?: (level: Types.PermissionLevel) => void;
}

export default function PermissionLevelSelector({
  level,
  onChange,
  ...selectProps
}: PermissionLevelSelectorProps) {
  return (
    <Select
      value={level}
      onChange={(value) => value && onChange?.(value as Types.PermissionLevel)}
      data={Object.keys(Types.PermissionLevel).map((level) => ({
        value: level,
        label: fmtSnakeCaseToUpperSpaceCase(level),
      }))}
      w={200}
      {...selectProps}
    />
  );
}
