import { useNavigate } from "react-router-dom";
import { ResourceComponents, UsableResource } from ".";
import { Button, Group, Stack, Text } from "@mantine/core";
import { ICONS } from "@/lib/icons";
import { usableResourcePath } from "@/lib/utils";

export default function ResourceNotFound({
  type,
}: {
  type: UsableResource | undefined;
}) {
  const nav = useNavigate();
  const Components = type && ResourceComponents[type];
  return (
    <Stack gap="md">
      {type && (
        <Group mb="xl">
          <Button
            variant="default"
            leftSection={<ICONS.Back size="1rem" />}
            onClick={() => nav("/" + usableResourcePath(type))}
          >
            Back
          </Button>
        </Group>
      )}
      <Group gap="lg">
        <div className="mt-1">
          {Components ? (
            <Components.Icon size="2em" />
          ) : (
            <ICONS.NotFound size="2rem" />
          )}
        </div>
        <Text fz="h1" ff="monospace">
          {type} {type && " - "} 404 Not Found
        </Text>
      </Group>
    </Stack>
  );
}
