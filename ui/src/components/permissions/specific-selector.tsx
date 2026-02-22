import { listsEqual } from "@/lib/utils";
import { UsableResource } from "@/resources";
import { MultiSelect, MultiSelectProps } from "@mantine/core";
import { Types } from "komodo_client";
import { useEffect, useState } from "react";

const ALL_PERMISSIONS_BY_TYPE: {
  [type: string]: Types.SpecificPermission[] | undefined;
} = {
  Swarm: [
    Types.SpecificPermission.Attach,
    Types.SpecificPermission.Inspect,
    Types.SpecificPermission.Logs,
  ],
  Server: [
    Types.SpecificPermission.Attach,
    Types.SpecificPermission.Inspect,
    Types.SpecificPermission.Logs,
    Types.SpecificPermission.Terminal,
    Types.SpecificPermission.Processes,
  ],
  Stack: [
    Types.SpecificPermission.Inspect,
    Types.SpecificPermission.Logs,
    Types.SpecificPermission.Terminal,
  ],
  Deployment: [
    Types.SpecificPermission.Inspect,
    Types.SpecificPermission.Logs,
    Types.SpecificPermission.Terminal,
  ],
  Build: [Types.SpecificPermission.Attach],
  Repo: [Types.SpecificPermission.Attach],
  Builder: [Types.SpecificPermission.Attach],
};

export interface SpecificPermissionSelectorProps extends Omit<
  MultiSelectProps,
  "onChange"
> {
  type: UsableResource;
  specific: Types.SpecificPermission[];
  onChange: (specific: Types.SpecificPermission[]) => void;
}

export default function SpecificPermissionSelector({
  type,
  specific,
  onChange,
  ...props
}: SpecificPermissionSelectorProps) {
  const [temp, setTemp] = useState(specific);
  useEffect(() => setTemp(specific), [specific.length]);
  return (
    <MultiSelect
      placeholder={!temp.length ? "Add specific permissions..." : undefined}
      value={temp}
      onChange={(specific) => setTemp(specific as Types.SpecificPermission[])}
      onBlur={() => !listsEqual(specific, temp) && onChange(temp)}
      onClear={() => onChange?.([])}
      data={ALL_PERMISSIONS_BY_TYPE[type]}
      clearable
      maw={500}
      {...props}
    />
  );
}
