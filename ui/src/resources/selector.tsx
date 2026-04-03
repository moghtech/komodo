import { Types } from "komodo_client";
import { ResourceComponents, UsableResource } from ".";
import {
  ActionIcon,
  Button,
  ButtonProps,
  Combobox,
  ComboboxProps,
  Group,
  InputWrapper,
  InputWrapperProps,
  Text,
} from "@mantine/core";
import { filterBySplit } from "@/lib/utils";
import { ChevronsUpDown } from "lucide-react";
import { fmtResourceType } from "@/lib/formatting";
import { ICONS } from "@/lib/icons";
import { useSearchCombobox } from "@/lib/hooks";

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
  wrapperProps?: InputWrapperProps;
  clearable?: boolean;
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
  wrapperProps,
  clearable = true,
  ...comboboxProps
}: ResourceSelectorProps) {
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

  const { search, setSearch, combobox } = useSearchCombobox();

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

  const Selector = (
    <Combobox
      store={combobox}
      width={300}
      position={position}
      onOptionSubmit={(id, props) => {
        onSelect?.(id);
        onOptionSubmit?.(id, props);
        combobox.closeDropdown();
      }}
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
          disabled={disabled}
          loading={!resources}
          {...targetProps}
        >
          <Group gap="xs" wrap="nowrap">
            <Components.Icon id={selected} />
            <Text className="text-ellipsis">
              {name || (placeholder ?? `Select ${fmtResourceType(type)}`)}
            </Text>
          </Group>
        </Button>
      </Combobox.Target>

      <Combobox.Dropdown>
        <Combobox.Search
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          leftSection={<ICONS.Search size="1rem" style={{ marginRight: 6 }} />}
          placeholder="search..."
        />
        <Combobox.Options mah={224} style={{ overflowY: "auto" }}>
          {filtered.map((resource) => (
            <Combobox.Option key={resource.id} value={resource.id}>
              <Group gap="xs">
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

  if (wrapperProps) {
    return <InputWrapper {...wrapperProps}>{Selector}</InputWrapper>;
  } else {
    return Selector;
  }
}
