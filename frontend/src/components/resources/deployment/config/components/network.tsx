import { ConfigItem } from "@components/config/util";
import { useRead } from "@lib/hooks";
import { Input } from "@ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { useState } from "react";

export const NetworkModeSelector = ({
  swarm_id,
  server_id,
  selected,
  onSelect,
  disabled,
}: {
  swarm_id: string | undefined;
  server_id: string | undefined;
  selected: string | undefined;
  onSelect: (type: string) => void;
  disabled: boolean;
}) => {
  const _networks =
    useRead(
      swarm_id ? "ListSwarmNetworks" : "ListDockerNetworks",
      { swarm: swarm_id, server: server_id! },
      { enabled: !!swarm_id || !!server_id }
    )
      .data?.filter((n) => n.name)
      .map((network) => network.name) ?? [];

  const [customMode, setCustomMode] = useState(false);

  const networks =
    !selected || _networks.includes(selected)
      ? _networks
      : [..._networks, selected];

  return (
    <ConfigItem
      label="Network Mode"
      boldLabel
      description="Choose the --network attached to container"
    >
      {customMode ? (
        <Input
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
      ) : (
        <Select
          value={selected || undefined}
          onValueChange={(value) => {
            if (value === "Custom") {
              setCustomMode(true);
              onSelect("");
            } else {
              onSelect(value);
            }
          }}
          disabled={disabled}
        >
          <SelectTrigger className="w-[200px]" disabled={disabled}>
            <SelectValue placeholder="Select Type" />
          </SelectTrigger>
          <SelectContent>
            {networks
              ?.filter((network) => network)
              .map((network) => (
                <SelectItem
                  key={network}
                  value={network!}
                  className="cursor-pointer"
                >
                  {network!}
                </SelectItem>
              ))}
            <SelectItem value="Custom" className="cursor-pointer">
              Custom
            </SelectItem>
          </SelectContent>
        </Select>
      )}
    </ConfigItem>
  );
};
