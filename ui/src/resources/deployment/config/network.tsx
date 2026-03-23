import { useRead } from "@/lib/hooks";
import { ConfigItem } from "@/ui/config/item";
import { Select, TextInput } from "@mantine/core";
import { useState } from "react";

export interface DeploymentNetworkSelectorProps {
  swarmId: string | undefined;
  serverId: string | undefined;
  selected: string | undefined;
  onSelect: (type: string) => void;
  disabled: boolean;
}

export default function DeploymentNetworkSelector({
  swarmId,
  serverId,
  selected,
  onSelect,
  disabled,
}: DeploymentNetworkSelectorProps) {
  const _networks =
    useRead(
      swarmId ? "ListSwarmNetworks" : "ListDockerNetworks",
      { swarm: swarmId, server: serverId! },
      { enabled: !!swarmId || !!serverId },
    )
      .data?.filter((network) => network.name)
      .map((network) => network.name as string) ?? [];

  const [customMode, setCustomMode] = useState(false);

  const networks =
    !selected || _networks.includes(selected)
      ? [..._networks, "Custom"]
      : [..._networks, selected, "Custom"];

  return (
    <ConfigItem
      label="Network Mode"
      description="Choose the --network attached to container"
    >
      {customMode && (
        <TextInput
          placeholder="Input custom network name"
          value={selected}
          onChange={(e) => onSelect(e.target.value)}
          className="max-w-[75%] lg:max-w-[400px]"
          onBlur={() => setCustomMode(false)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              setCustomMode(false);
            }
          }}
          autoFocus
        />
      )}
      {!customMode && (
        <Select
          value={selected || undefined}
          onChange={(value) => {
            if (value === "Custom") {
              setCustomMode(true);
              onSelect("");
            } else if (value) {
              onSelect(value);
            }
          }}
          disabled={disabled}
          data={networks}
          w={{ base: "100%", xs: "fit-content" }}
          searchable
        />
      )}
    </ConfigItem>
  );
}
