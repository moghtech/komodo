import { useRead, useSearchCombobox } from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import {
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

export interface StackServiceSelectorProps extends ComboboxProps {
  stackId: string;
  selected: string | undefined;
  onSelect?: (name: string) => void;
  disabled?: boolean;
  placeholder?: string;
  state?: Types.ContainerStateStatusEnum;
  targetProps?: ButtonProps;
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
  ...comboboxProps
}: StackServiceSelectorProps) {
  const services = useRead("ListStackServices", {
    stack: stackId,
  }).data?.filter((service) => !state || service?.container?.state === state);
  const firstService = services?.[0].service;
  useEffect(() => {
    firstService && onSelect?.(firstService);
  }, [firstService]);
  const name = services?.find((s) => s.service === selected)?.service;
  const container = services?.find((s) => s.service === selected)?.container;

  const { search, setSearch, combobox } = useSearchCombobox();

  if (!services) return null;

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
          maw={350}
          justify="space-between"
          disabled={disabled}
          rightSection={<ChevronsUpDown size="0.9rem" />}
          onClick={() => combobox.toggleDropdown()}
          {...targetProps}
        >
          <Group gap="xs">
            {container && (
              <DOCKER_LINK_ICONS.Container
                serverId={container.server_id!}
                name={container.name}
              />
            )}
            <Text>{name || (placeholder ?? "Select container")}</Text>
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
          {!search && <Combobox.Option value="None">None</Combobox.Option>}
          {filtered.map((service) => (
            <Combobox.Option key={service.service} value={service.service}>
              <Group>
                {service.container && (
                  <DOCKER_LINK_ICONS.Container
                    serverId={service.container.server_id!}
                    name={service.container.name}
                  />
                )}
                {service.service}
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
