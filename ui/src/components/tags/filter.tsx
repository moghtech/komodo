import { useEffect, useState } from "react";
import { useRead, useShiftKeyListener, useTags } from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import {
  ActionIcon,
  Badge,
  Box,
  Button,
  Center,
  Combobox,
  Group,
  Text,
  useCombobox,
} from "@mantine/core";
import Tags from ".";
import { ICONS } from "@/lib/icons";
import { CircleMinus, X } from "lucide-react";

export default function TagsFilter() {
  const [search, setSearch] = useState("");
  const { tags, add_tag, remove_tag, clear_tags } = useTags();
  const other_tags = useRead("ListTags", {}).data?.filter(
    (tag) => !tags.includes(tag._id!.$oid),
  );
  const filtered = filterBySplit(other_tags, search, (item) => item.name);
  useShiftKeyListener("T", () => combobox.openDropdown());
  useShiftKeyListener("C", () => clear_tags());
  const combobox = useCombobox({
    onDropdownClose: () => {
      combobox.resetSelectedOption();
      combobox.focusTarget();
      setSearch("");
    },
    onDropdownOpen: () => {
      combobox.focusSearchInput();
    },
  });
  useEffect(() => {
    combobox.selectFirstOption();
  }, [search]);

  return (
    <Group gap="xs">
      {tags.length > 0 && (
        <ActionIcon color="red" onClick={clear_tags} opacity={0.7}>
          <X size="1rem" />{" "}
        </ActionIcon>
      )}

      <Tags
        tag_ids={tags}
        onBadgeClick={remove_tag}
        icon={<CircleMinus size="1rem" />}
      />

      <Combobox
        store={combobox}
        width={230}
        position="bottom-end"
        onOptionSubmit={(tag_id) => {
          add_tag(tag_id);
          setSearch("");
        }}
      >
        <Combobox.Target>
          <Button
            variant="default"
            pl="0.4rem"
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
          >
            Tag Filter
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
              <Combobox.Option key={tag._id?.$oid} value={tag._id?.$oid!}>
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
    </Group>
  );
}
