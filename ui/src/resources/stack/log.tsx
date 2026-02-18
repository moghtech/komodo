import LogSection, { LogSectionProps } from "@/components/log-section";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { MultiSelect, Select } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { useStack } from ".";

export interface StackLogProps extends Omit<LogSectionProps, "target"> {
  id: string;
}

export default function StackLog({ id, ...props }: StackLogProps) {
  const isSwarm = !!useStack(id)?.info.swarm_id;
  const [selectedServices, setServices] = useLocalStorage<string[]>({
    key: `stack-${id}-log-services-v1`,
    defaultValue: [],
  });
  const allServices = useRead("ListStackServices", { stack: id }).data?.map(
    (s) => s.service,
  );
  return (
    <LogSection
      target={{
        type: "Stack",
        stackId: id,
        services: isSwarm
          ? [selectedServices[0] ?? allServices?.[0]]
          : selectedServices,
      }}
      extraController={
        isSwarm ? (
          <Select
            leftSection={<ICONS.Service size="1rem" />}
            placeholder={selectedServices?.length ? undefined : "All services"}
            value={selectedServices[0] ?? allServices?.[0]}
            data={allServices}
            onChange={(service) => service && setServices([service])}
            w={{ base: 200, lg: 250 }}
            searchable
          />
        ) : (
          <MultiSelect
            leftSection={<ICONS.Service size="1rem" />}
            placeholder={selectedServices?.length ? undefined : "All services"}
            value={selectedServices}
            data={allServices}
            onChange={setServices}
            styles={{ inputField: { width: 130 } }}
            searchable
            clearable
          />
        )
      }
      {...props}
    />
  );
}
