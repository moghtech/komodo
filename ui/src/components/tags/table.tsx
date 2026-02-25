import { Group } from "@mantine/core";
import { useTags } from "@/lib/hooks";
import Tags from ".";

export default function TableTags({ tagIds }: { tagIds: string[] }) {
  const { toggle_tag } = useTags();
  return (
    <Group gap="xs" wrap="nowrap">
      <Tags tagIds={tagIds} onBadgeClick={toggle_tag} />
    </Group>
  );
}
