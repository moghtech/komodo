import { Select, SelectProps, TextInput } from "@mantine/core";
import { useState } from "react";

export interface OrganizationSelectorProps extends Omit<
  SelectProps,
  "onSelect"
> {
  organizations: string[];
  selected: string;
  onSelect: (organization: string) => void;
  disabled: boolean;
  showLabel?: boolean;
}

export default function OrganizationSelector({
  organizations,
  selected,
  onSelect,
  disabled,
  showLabel,
  ...selectorProps
}: OrganizationSelectorProps) {
  const [customMode, setCustomMode] = useState(false);

  if (customMode) {
    return (
      <TextInput
        label="Organization"
        placeholder="Input custom organization name"
        w={{ base: "85%", lg: 400 }}
        value={selected}
        onChange={(e) => onSelect(e.target.value)}
        onBlur={() => setCustomMode(false)}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            setCustomMode(false);
          }
        }}
        autoFocus
      />
    );
  }

  const orgSet = new Set<string>();
  selected && orgSet.add(selected);
  for (const org of organizations) {
    orgSet.add(org);
  }
  const orgs = ["None", ...orgSet, "Custom"];
  orgs.sort();

  return (
    <Select
      placeholder="Select Organization"
      label={showLabel && "Organization"}
      value={selected}
      disabled={disabled}
      data={orgs}
      onChange={(value) => {
        if (value === "Custom") {
          onSelect("");
          setCustomMode(true);
        } else if (value === "None") {
          onSelect("");
        } else if (value) {
          onSelect(value);
        }
      }}
      searchable
      {...selectorProps}
    />
  );
}
