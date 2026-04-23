import { useEffect } from "react";
import { ChevronsUpDown } from "lucide-react";
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
import { useRead, useSearchCombobox } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { filterBySplit } from "@/lib/utils";
import { DOCKER_LINK_ICONS } from "@/components/docker/link";

export interface ContainerSelectorProps extends ComboboxProps {
  serverId: string;
  selected: string | undefined;
  onSelect?: (name: string) => void;
  disabled?: boolean;
  placeholder?: string;
  state?: Types.ContainerStateStatusEnum;
  targetProps?: ButtonProps;
  clearable?: boolean;
}

export default function ContainerSelector({
  serverId,
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
}: ContainerSelectorProps) {
  const containers = useRead("ListDockerContainers", {
    server: serverId,
  }).data?.filter((container) => !state || container.state === state);

  const firstContainer = containers?.[0]?.name;
  useEffect(() => {
    !clearable && firstContainer && !selected && onSelect?.(firstContainer);
  }, [firstContainer]);

  const name = containers?.find((r) => r.name === selected)?.name;

  const { search, setSearch, combobox } = useSearchCombobox();

  const filtered = filterBySplit(containers, search, (item) => item.name).sort(
    (a, b) => {
      if (a.name > b.name) {
        return 1;
      } else if (a.name < b.name) {
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
          rightSection={
            <Group gap="xs" ml="sm" wrap="nowrap">
              {clearable && (
                <ActionIcon
                  size="sm"
                  variant="filled"
                  color="red"
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
          {...targetProps}
        >
          <Group gap="xs" wrap="nowrap">
            <DOCKER_LINK_ICONS.Container serverId={serverId} name={selected} />
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
          {filtered.map((container) => (
            <Combobox.Option key={container.name} value={container.name}>
              <Group gap="xs">
                <DOCKER_LINK_ICONS.Container
                  serverId={container.server_id!}
                  name={container.name}
                />
                <Text>{container.name}</Text>
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
