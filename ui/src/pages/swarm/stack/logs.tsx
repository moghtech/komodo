import LogSection from "@/components/log-section";
import { SectionProps } from "mogh_ui";
import { Select } from "@mantine/core";
import { Types } from "komodo_client";
import { useState } from "react";

export interface SwarmStackLogsSectionProps extends SectionProps {
  swarmId: string;
  stack: Types.SwarmStack;
  disabled: boolean;
}

export default function SwarmStackLogsSection({
  swarmId,
  stack,
  disabled,
  ...sectionProps
}: SwarmStackLogsSectionProps) {
  const [service, setService] = useState(stack.Services[0].Name ?? "");
  return (
    <LogSection
      {...sectionProps}
      target={{ type: "SwarmService", swarmId, service }}
      disabled={disabled}
      extraController={
        <Select
          placeholder="Select service"
          value={service}
          onChange={(service) => service && setService(service)}
          data={stack.Services.map((s) => s.Name).filter((s) => s!) as string[]}
          searchable
        />
      }
    />
  );
}
