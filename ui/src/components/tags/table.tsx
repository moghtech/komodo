import { Group } from "@mantine/core";
import { useTags } from "@/lib/hooks";
import Tags from ".";

export default function TableTags({ tag_ids }: { tag_ids: string[] }) {
  const { toggle_tag } = useTags();
  return (
    <Group gap="xs" wrap="nowrap">
      <Tags tag_ids={tag_ids} onBadgeClick={toggle_tag} />
    </Group>
  );
}
