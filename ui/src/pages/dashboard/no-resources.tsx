import { useNoResources, useUser } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { Group, Text } from "@mantine/core";

export default function DashboardNoResources() {
  const noResources = useNoResources();
  const user = useUser().data!;
  return (
    noResources && (
      <Group gap="sm" opacity={0.6}>
        <ICONS.Alert size="1rem" />
        <Text fz="lg">
          No resources found.{" "}
          {user.admin
            ? "To get started, create a server."
            : "Contact an admin for access to resources."}
        </Text>
      </Group>
    )
  );
}
