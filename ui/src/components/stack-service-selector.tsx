import { useRead, useSearchCombobox } from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import {
  ActionIcon,
  Button,
  ButtonProps,
  Combobox,
  ComboboxProps,
  Group,
  Text,
} from "@mantine/core";
import { Types } from "komodo_client";
import { ChevronsUpDown } from "lucide-react";
import { useEffect } from "react";
import { DOCKER_LINK_ICONS } from "./docker/link";
import { ICONS } from "@/theme/icons";
import {
  colorByIntention,
  containerStateIntention,
  swarmStateIntention,
} from "@/lib/color";
import { SWARM_LINK_ICONS } from "./swarm/link";
import { useStack } from "@/resources/stack";

export interface StackServiceSelectorProps extends ComboboxProps {
  stackId: string;
  selected: string | undefined;
  onSelect?: (name: string) => void;
  disabled?: boolean;
  placeholder?: string;
  state?: Types.ContainerStateStatusEnum;
  targetProps?: ButtonProps;
  clearable?: boolean;
}

export default function StackServiceSelector({
  stackId,
  selected,
  onSelect,
  disabled,
  placeholder,
  state,
  position = "bottom-start",
  onOptionSubmit,
  targetProps,
  clearable,
  ...comboboxProps
}: StackServiceSelectorProps) {
  const stack = useStack(stackId);
  const services = useRead("ListStackServices", {
    stack: stackId,
  }).data?.filter((service) => !state || service?.container?.state === state);

  const firstService = services?.[0]?.service;
  useEffect(() => {
    !clearable && firstService && !selected && onSelect?.(firstService);
  }, [firstService]);

  const selectedService = services?.find((s) => s.service === selected);
  const name = selectedService?.service;
  const container = selectedService?.container;
  const swarmService = selectedService?.swarm_service;

  const intention = !selectedService
    ? "None"
    : swarmService?.State
      ? swarmStateIntention(swarmService.State)
      : containerStateIntention(
          container?.state ?? Types.ContainerStateStatusEnum.Empty,
        );

  const { search, setSearch, combobox } = useSearchCombobox();

  const filtered = filterBySplit(services, search, (item) => item.service).sort(
    (a, b) => {
      if (a.service > b.service) {
        return 1;
      } else if (a.service < b.service) {
        return -1;
      } else {
        return 0;
      }
    },
  );

  return (
    <Combobox
      store={combobox}
      width={300}
      onOptionSubmit={(name, props) => {
        onSelect?.(name);
        onOptionSubmit?.(name, props);
        combobox.closeDropdown();
      }}
      position={position}
      {...comboboxProps}
    >
      <Combobox.Target>
        <Button
          justify="space-between"
          w="fit-content"
          maw="100%"
          rightSection={
            <Group gap="xs" ml="sm" wrap="nowrap">
              {clearable && (
                <ActionIcon
                  size="sm"
                  variant="subtle"
                  color="dimmed"
                  onClick={(e) => {
                    e.stopPropagation();
                    onSelect?.("");
                  }}
                  disabled={disabled || !selected}
                >
                  <ICONS.Clear size="0.8rem" />
                </ActionIcon>
              )}
              <ChevronsUpDown size="1rem" />
            </Group>
          }
          onClick={() => combobox.toggleDropdown()}
          disabled={!stackId || disabled}
          loading={!!stackId && !services}
          {...targetProps}
        >
          <Group gap="xs" wrap="nowrap">
            <ICONS.Service size="1rem" color={colorByIntention(intention)} />
            <Text className="text-ellipsis">
              {name || (placeholder ?? "Select service")}
            </Text>
          </Group>
        </Button>
      </Combobox.Target>

      <Combobox.Dropdown>
        <Combobox.Search
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          leftSection={<ICONS.Search size="1rem" style={{ marginRight: 6 }} />}
          placeholder="Search"
        />
        <Combobox.Options mah={224} style={{ overflowY: "auto" }}>
          {filtered.map((service) => (
            <Combobox.Option key={service.service} value={service.service}>
              <Group gap="xs">
                {service.container && (
                  <DOCKER_LINK_ICONS.Container
                    serverId={service.container.server_id!}
                    name={service.container.name}
                  />
                )}
                {service.swarm_service && (
                  <SWARM_LINK_ICONS.Service
                    swarmId={stack?.info.swarm_id}
                    resourceId={service.swarm_service.ID}
                  />
                )}
                <Text>{service.service}</Text>
              </Group>
            </Combobox.Option>
          ))}
          {filtered.length === 0 && (
            <Combobox.Empty>No results.</Combobox.Empty>
          )}
        </Combobox.Options>
      </Combobox.Dropdown>
    </Combobox>
  );
}
