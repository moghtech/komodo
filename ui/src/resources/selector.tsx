import { Types } from "komodo_client";
import { ResourceComponents, UsableResource } from ".";
import {
  Button,
  ButtonProps,
  Combobox,
  ComboboxProps,
  Group,
  Text,
  useCombobox,
} from "@mantine/core";
import { useEffect, useState } from "react";
import { filterBySplit } from "@/lib/utils";
import { ChevronsUpDown } from "lucide-react";
import { fmtResourceType } from "@/lib/formatting";
import { ICONS } from "@/lib/icons";

export interface ResourceSelectorProps extends ComboboxProps {
  type: UsableResource;
  selected: string | undefined;
  templates?: Types.TemplatesQueryBehavior;
  onSelect?: (id: string) => void;
  disabled?: boolean;
  placeholder?: string;
  state?: unknown;
  excludeIds?: string[];
  targetProps?: ButtonProps;
}

export default function ResourceSelector({
  type,
  selected,
  onSelect,
  disabled,
  templates = Types.TemplatesQueryBehavior.Exclude,
  placeholder,
  state,
  excludeIds,
  onOptionSubmit,
  position = "bottom-start",
  targetProps,
  ...comboboxProps
}: ResourceSelectorProps) {
  const [search, setSearch] = useState("");
  const templateFilterFn =
    templates === Types.TemplatesQueryBehavior.Exclude
      ? (r: Types.ResourceListItem<unknown>) => !r.template
      : templates === Types.TemplatesQueryBehavior.Only
        ? (r: Types.ResourceListItem<unknown>) => r.template
        : () => true;
  const Components = ResourceComponents[type];
  const resources = Components.useList()?.filter(
    (r) =>
      templateFilterFn(r) &&
      (!state || (r.info as any).state === state) &&
      (!excludeIds || r.id === selected || !excludeIds?.includes(r.id)),
  );
  const name = resources?.find((r) => r.id === selected)?.name;

  const combobox = useCombobox({
    onDropdownOpen: () => {
      combobox.focusSearchInput();
    },
    onDropdownClose: () => {
      combobox.resetSelectedOption();
      combobox.focusTarget();
      setSearch("");
    },
  });
  useEffect(() => {
    combobox.selectFirstOption();
  }, [search]);

  if (!resources) return null;

  const filtered = filterBySplit(resources, search, (item) => item.name).sort(
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
      width={260}
      onOptionSubmit={(_id, props) => {
        const id = _id === "None" ? "" : _id;
        onSelect?.(id);
        onOptionSubmit?.(id, props);
        combobox.closeDropdown();
      }}
      position={position}
      {...comboboxProps}
    >
      <Combobox.Target>
        <Button
          justify="space-between"
          rightSection={<ChevronsUpDown size="1rem" />}
          onClick={() => combobox.toggleDropdown()}
          disabled={disabled}
          w="fit-content"
          maw={{ base: 200, lg: 300 }}
          {...targetProps}
        >
          <Group gap="xs">
            <Components.Icon id={selected} />
            <Text>
              {name || (placeholder ?? `Select ${fmtResourceType(type)}`)}
            </Text>
          </Group>
        </Button>
      </Combobox.Target>

      <Combobox.Dropdown>
        <Combobox.Search
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          leftSection={<ICONS.Search size="1rem" />}
          placeholder="Search"
          styles={{
            section: {
              marginRight: 4,
            },
          }}
        />
        <Combobox.Options mah={224} style={{ overflowY: "auto" }}>
          <Combobox.Option value="None">None</Combobox.Option>
          {filtered.map((resource) => (
            <Combobox.Option key={resource.id} value={resource.id}>
              <Group>
                <Components.Icon id={resource.id} />
                <Text>{resource.name}</Text>
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
