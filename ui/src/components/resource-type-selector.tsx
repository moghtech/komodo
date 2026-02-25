import { RESOURCE_TARGETS, UsableResource } from "@/resources";
import { Select, SelectProps } from "@mantine/core";

export interface ResourceTypeSelectorProps extends Omit<
  SelectProps,
  "onChange"
> {
  value: UsableResource | null;
  onChange?: (type: UsableResource | null) => void;
}

export default function ResourceTypeSelector({
  value,
  onChange,
  ...selectProps
}: ResourceTypeSelectorProps) {
  return (
    <Select
      placeholder="Select resource type"
      value={value}
      onChange={(type) => {
        onChange?.(type as UsableResource);
      }}
      data={RESOURCE_TARGETS}
      searchable
      clearable
      {...selectProps}
    />
  );
}
