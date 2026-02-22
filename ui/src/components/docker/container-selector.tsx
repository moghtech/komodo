import { useEffect } from "react";
import { ChevronsUpDown } from "lucide-react";
import {
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
  ...comboboxProps
}: ContainerSelectorProps) {
  const containers = useRead("ListDockerContainers", {
    server: serverId,
  }).data?.filter((container) => !state || container.state === state);
  const firstContainer = containers?.[0].name;
  useEffect(() => {
    firstContainer && onSelect?.(firstContainer);
  }, [firstContainer]);
  const name = containers?.find((r) => r.name === selected)?.name;

  const { search, setSearch, combobox } = useSearchCombobox();

  if (!containers) return null;

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
          rightSection={<ChevronsUpDown size="0.9rem" />}
          onClick={() => combobox.toggleDropdown()}
          {...targetProps}
        >
          <Group gap="xs">
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
          {!search && <Combobox.Option value="None">None</Combobox.Option>}
          {filtered.map((container) => (
            <Combobox.Option key={container.name} value={container.name}>
              <Group>
                <DOCKER_LINK_ICONS.Container
                  serverId={container.server_id!}
                  name={container.name}
                />
                {container.name}
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
