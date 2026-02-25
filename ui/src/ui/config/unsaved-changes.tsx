import { ICONS } from "@/theme/icons";
import { Group } from "@mantine/core";

export default function UnsavedChanges({ fullWidth }: { fullWidth?: boolean }) {
  return (
    <Group
      gap="xs"
      wrap="nowrap"
      justify="space-evenly"
      w={fullWidth ? "100%" : "fit-content"}
    >
      <ICONS.Alert size="1rem" />
      Unsaved changes
      <ICONS.Alert size="1rem" />
    </Group>
  );
}
