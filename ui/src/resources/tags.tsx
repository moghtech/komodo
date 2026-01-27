import Tags from "@/components/tags";
import { tagColor } from "@/lib/color";
import {
  useInvalidate,
  useRead,
  useShiftKeyListener,
  useWrite,
} from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { filterBySplit } from "@/lib/utils";
import {
  ActionIcon,
  Box,
  Center,
  Combobox,
  Divider,
  Group,
  Stack,
  Text,
  useCombobox,
} from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { MinusCircle } from "lucide-react";
import { useEffect, useState } from "react";
import { ResourceComponents, UsableResourceTarget } from ".";

export const ResourceTags = ({
  target,
  click_to_delete,
  disabled,
}: {
  target: UsableResourceTarget;
  click_to_delete?: boolean;
  disabled?: boolean;
}) => {
  const inv = useInvalidate();
  const { type, id } = target;
  const resource = useRead(`List${type}s`, {}).data?.find((d) => d.id === id);
  const { mutate } = useWrite("UpdateResourceMeta", {
    onSuccess: () => {
      inv([`List${type}s`]);
      notifications.show({ message: "Removed tag" });
    },
  });
  return (
    <Tags
      tag_ids={resource?.tags}
      onBadgeClick={(tag_id) => {
        if (!click_to_delete) return;
        if (disabled) return;
        mutate({
          target,
          tags: resource!.tags.filter((tag) => tag !== tag_id),
        });
      }}
      icon={!disabled && click_to_delete && <MinusCircle size="1rem" />}
    />
  );
};

export const AddResourceTags = (target: UsableResourceTarget) => {
  const { type, id } = target;
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
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

  const inv = useInvalidate();

  const Components = ResourceComponents[type];
  const resource = Components.useListItem(id);

  useShiftKeyListener(
    "T",
    () => !combobox.dropdownOpened && combobox.openDropdown(),
  );

  const all_tags = useRead("ListTags", {}).data ?? [];
  const all_tag_names = all_tags.map((tag) => tag.name);

  const { mutate: update } = useWrite("UpdateResourceMeta", {
    onSuccess: () => {
      inv([`List${type}s`]);
      setOpen(false);
      notifications.show({ message: `Added tag ${search}` });
    },
  });

  const { mutateAsync: create } = useWrite("CreateTag", {
    onSuccess: () => inv([`ListTags`]),
  });

  useEffect(() => {
    if (open) setSearch("");
  }, [open]);

  const createTag = async (name: string) => {
    if (!name) {
      notifications.show({ message: "Must provide tag name in input" });
      return;
    }
    const tag = await create({ name });
    update({
      target,
      tags: [...(resource?.tags ?? []), tag._id!.$oid],
    });
    setOpen(false);
  };

  useEffect(() => {
    combobox.selectFirstOption();
  }, [search]);

  if (!resource) {
    return null;
  }

  const filtered = filterBySplit(
    all_tags.filter((tag) => !resource?.tags.includes(tag._id!.$oid)),
    search,
    (item) => item.name,
  ).sort((a, b) => {
    if (a.name > b.name) {
      return 1;
    } else if (a.name < b.name) {
      return -1;
    } else {
      return 0;
    }
  });

  return (
    <Combobox
      store={combobox}
      width={230}
      position="bottom-start"
      onOptionSubmit={(tag_id) => {
        if (tag_id === "Create") {
          createTag(search);
        } else {
          update({
            target,
            tags: [...(resource?.tags ?? []), tag_id],
          });
        }
      }}
    >
      <Combobox.Target>
        <ActionIcon
          variant="default"
          c="inherit"
          onClick={() => combobox.toggleDropdown()}
        >
          <ICONS.Create size="1rem" />
        </ActionIcon>
      </Combobox.Target>

      <Combobox.Dropdown>
        <Combobox.Search
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          leftSection={<ICONS.Search size="1rem" />}
          placeholder="Search / Create"
          styles={{
            section: {
              marginRight: 4,
            },
          }}
        />
        <Combobox.Options renderRoot={(props) => <Stack gap="xs" {...props} />}>
          {filtered.map((tag) => (
            <Combobox.Option key={tag._id?.$oid} value={tag.name}>
              <Group justify="space-between">
                <Text>{tag.name}</Text>
                <Box w={25} h={25} bg={tagColor(tag.color)} bdrs="md" />
              </Group>
            </Combobox.Option>
          ))}

          <Divider />

          <Combobox.Option
            value="Create"
            disabled={!search || all_tag_names.includes(search)}
          >
            <Center>
              <ICONS.Create size="1rem" />
              Create Tag
            </Center>
          </Combobox.Option>
        </Combobox.Options>
      </Combobox.Dropdown>
    </Combobox>
  );
};
