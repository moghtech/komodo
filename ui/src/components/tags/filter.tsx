import { useRead, useTags } from "@/lib/hooks";
import { ActionIcon, Group, Stack } from "@mantine/core";
import Tags from ".";
import TagSelector from "./selector";
import { ICONS } from "@/lib/icons";
import { useShiftKeyListener } from "mogh_ui";

export default function TagsFilter() {
  const { tags, add_tag, remove_tag, clear_tags } = useTags();
  const otherTags = useRead("ListTags", {}).data?.filter(
    (tag) => !tags.includes(tag._id!.$oid),
  );
  useShiftKeyListener("C", () => clear_tags());
  return (
    <>
      <Group gap="xs" visibleFrom="xs">
        {tags.length > 0 && (
          <ActionIcon color="red" onClick={clear_tags} opacity={0.7}>
            <ICONS.Clear size="1rem" />{" "}
          </ActionIcon>
        )}

        <Tags
          tagIds={tags}
          onBadgeClick={remove_tag}
          icon={<ICONS.Remove size="1rem" />}
        />

        <TagSelector
          title="Tag Filter"
          tags={otherTags}
          onSelect={(tagId) => add_tag(tagId)}
          shiftKey="T"
          position="bottom-end"
        />
      </Group>
      <Stack hiddenFrom="xs" w="100%">
        <TagSelector
          title="Tag Filter"
          tags={otherTags}
          onSelect={(tagId) => add_tag(tagId)}
          shiftKey="T"
          position="bottom-end"
        />

        {tags.length > 0 && (
          <Group gap="xs">
            <Tags
              tagIds={tags}
              onBadgeClick={remove_tag}
              icon={<ICONS.Remove size="1rem" />}
            />

            {tags.length > 0 && (
              <ActionIcon color="red" onClick={clear_tags} opacity={0.7}>
                <ICONS.Clear size="1rem" />{" "}
              </ActionIcon>
            )}
          </Group>
        )}
      </Stack>
    </>
  );
}
