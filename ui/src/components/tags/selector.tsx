import { useShiftKeyListener } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { filterBySplit } from "@/lib/utils";
import {
  Badge,
  Box,
  Button,
  Center,
  Combobox,
  ComboboxProps,
  Group,
  Text,
  useCombobox,
} from "@mantine/core";
import { Types } from "komodo_client";
import { useEffect, useState } from "react";

export interface TagSelectorProps extends ComboboxProps {
  title: string;
  tags?: Types.Tag[];
  onSelect?: (tagId: string) => void;
  shiftKey?: string;
  useName?: boolean;
}

export default function TagSelector({
  title,
  tags,
  onSelect,
  shiftKey,
  useName,
  disabled,
  ...comboboxProps
}: TagSelectorProps) {
  const [search, setSearch] = useState("");
  const filtered = filterBySplit(tags, search, (item) => item.name);
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
  useShiftKeyListener(
    shiftKey ?? "",
    () => shiftKey && combobox.openDropdown(),
  );

  return (
    <Combobox
      store={combobox}
      width={230}
      onOptionSubmit={(tag) => {
        onSelect?.(tag);
        setSearch("");
      }}
      disabled={disabled}
      {...comboboxProps}
    >
      <Combobox.Target>
        <Button
          variant="filled"
          color="accent.1"
          pl="0.4rem"
          className="bordered-heavy"
          fw="normal"
          leftSection={
            <Badge
              radius="sm"
              px="0.3rem"
              py="0.3rem"
              color="accent"
              c="dimmed"
              h="fit-content"
            >
              <Center>
                <ICONS.Tag size="0.7rem" />
              </Center>
            </Badge>
          }
          onClick={() => combobox.toggleDropdown()}
          disabled={disabled}
          loading={!tags}
        >
          {title}
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
          {filtered.map((tag) => (
            <Combobox.Option
              key={tag._id?.$oid}
              value={useName ? tag.name : tag._id?.$oid!}
            >
              <Group justify="space-between">
                <Text>{tag.name}</Text>
                <Box w={25} h={25} bg={"Tag" + tag.color} bdrs="md" />
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
