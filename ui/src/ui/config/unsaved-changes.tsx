import { ICONS } from "@/theme/icons";
import { Group } from "@mantine/core";

export default function UnsavedChanges() {
  return (
    <Group>
      <ICONS.Alert size="1rem" />
      Unsaved changes
      <ICONS.Alert size="1rem" />
    </Group>
  );
}
