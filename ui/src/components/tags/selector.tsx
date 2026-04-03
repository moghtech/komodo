import {
  useInvalidate,
  useRead,
  useSearchCombobox,
  useShiftKeyListener,
  useWrite,
} from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { filterBySplit } from "@/lib/utils";
import {
  Badge,
  Box,
  Button,
  Center,
  Combobox,
  ComboboxProps,
  Divider,
  Group,
  Text,
} from "@mantine/core";
import { Types } from "komodo_client";
import { notifications } from "@mantine/notifications";

export interface TagSelectorProps extends ComboboxProps {
  title: string;
  tags?: Types.Tag[];
  onSelect?: (tagId: string) => void;
  shiftKey?: string;
  useName?: boolean;
  canCreate?: boolean;
}

export default function TagSelector({
  title,
  tags,
  onSelect,
  shiftKey,
  useName,
  disabled,
  canCreate,
  ...comboboxProps
}: TagSelectorProps) {
  const { search, setSearch, combobox } = useSearchCombobox();
  const filtered = filterBySplit(tags, search, (item) => item.name);

  useShiftKeyListener(
    shiftKey ?? "",
    () => shiftKey && combobox.openDropdown(),
  );

  const inv = useInvalidate();
  const { mutateAsync: create } = useWrite("CreateTag", {
    onSuccess: () => {
      inv([`ListTags`]);
      notifications.show({ message: `Created tag ${search}`, color: "green" });
    },
  });

  const createTag = async (name: string) => {
    if (!name) {
      notifications.show({ message: "Must provide tag name in input" });
      return;
    }
    const tag = await create({ name });
    onSelect?.(useName ? tag.name : tag._id?.$oid!);
  };

  const allTags = useRead("ListTags", {}).data ?? [];
  const allTagNames = allTags.map((tag) => tag.name);

  return (
    <Combobox
      store={combobox}
      width={260}
      onOptionSubmit={(tag) => {
        if (tag === "__CREATE__" && search) {
          createTag(search);
        } else {
          onSelect?.(tag);
        }
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
          justify="start"
          w={{ base: "100%", xs: "138" }}
          fw="normal"
          leftSection={
            <Badge
              radius="sm"
              px="0.3rem"
              py="0.3rem"
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
                <Box w={25} h={25} bg={`Tag${tag.color}.9`} bdrs="md" />
              </Group>
            </Combobox.Option>
          ))}
          {!canCreate && filtered.length === 0 && (
            <Combobox.Empty>No results.</Combobox.Empty>
          )}
          {canCreate && (
            <>
              <Divider />
              <Combobox.Option
                value="__CREATE__"
                disabled={!search || allTagNames.includes(search)}
              >
                <Center>
                  <ICONS.Create size="1rem" />
                  Create Tag
                </Center>
              </Combobox.Option>
            </>
          )}
        </Combobox.Options>
      </Combobox.Dropdown>
    </Combobox>
  );
}
