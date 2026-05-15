import { fmtUpperCamelcase } from "mogh_ui";
import { ConfigItem } from "mogh_ui";
import { Select } from "@mantine/core";
import { Types } from "komodo_client";

export interface DeploymentRestartSelectorProps {
  selected: Types.RestartMode | undefined;
  set: (input: Partial<Types.DeploymentConfig>) => void;
  disabled: boolean;
}

export default function DeploymentRestartSelector({
  selected,
  set,
  disabled,
}: DeploymentRestartSelectorProps) {
  return (
    <ConfigItem
      label="Restart Mode"
      description="Configure the --restart behavior."
    >
      <Select
        value={selected || undefined}
        onChange={(restart) =>
          restart && set({ restart: restart as Types.RestartMode })
        }
        disabled={disabled}
        placeholder="Select Mode"
        data={Object.entries(Types.RestartMode).map(([label, value]) => ({
          label: fmtUpperCamelcase(label),
          value,
        }))}
        tt="capitalize"
        w={{ base: "100%", xs: "fit-content" }}
      />
    </ConfigItem>
  );
}
