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
import { ChevronsUpDown } from "lucide-react";
import { fmtResourceType } from "@/lib/formatting";
import { ICONS } from "@/lib/icons";
import { useDebounce, useSearchCombobox } from "mogh_ui";
import { keepPreviousData } from "@tanstack/react-query";
import { useMemo } from "react";
import { useRead } from "@/lib/hooks";

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
  const { search, setSearch, combobox } = useSearchCombobox();
  const debouncedSearch = useDebounce(search, 200);
  const terms = useMemo(
    () =>
      debouncedSearch
        .split(" ")
        .map((item) => item.trim())
        .filter((item) => !!item),
    [debouncedSearch],
  );

  const Components = ResourceComponents[type];
  const selectedResource = Components.useListItem(selected);

  const { data: __resources } = useRead(
    `List${type}s`,
    {
      query: { templates, terms },
      limit: 10,
    },
    // Keep the previous options visible while the
    // next search fetches, to prevent flashing.
    { placeholderData: keepPreviousData },
  );

  const resources = useMemo(
    () =>
      (__resources?.filter(
        (r) =>
          (!state || (r.info as any).state === state) &&
          (!excludeIds || r.id === selected || !excludeIds?.includes(r.id)),
      ) ?? []) as Array<Types.ResourceListItem<any>>,
    [__resources],
  );

  const name = selectedResource?.name;

  if (
    terms.length === 0 &&
    selectedResource &&
    resources.every((resource) => resource.id !== selectedResource.id)
  ) {
    resources.push(selectedResource);
  }

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
          {/* {isFetching && (
            <Center pt="xs">
              <Loader size="sm" />
            </Center>
          )} */}
          {resources.map((resource) => (
            <Combobox.Option key={resource.id} value={resource.id}>
              <Group gap="xs">
                <Components.Icon id={resource.id} />
                <Text>{resource.name}</Text>
              </Group>
            </Combobox.Option>
          ))}
          {resources.length === 0 && (
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
