import { Link, useNavigate } from "react-router-dom";
import { ResourceComponents, UsableResource } from ".";
import { Button, Flex, Group, Stack, Text } from "@mantine/core";
import { usableResourcePath } from "@/lib/utils";
import { TemplateMarker } from "@/components/template-marker";
import { ICONS } from "@/lib/icons";

export const ResourceNameSimple = ({
  type,
  id,
}: {
  type: UsableResource;
  id: string;
}) => {
  const Components = ResourceComponents[type];
  const name = Components.useListItem(id)?.name ?? "unknown";
  return <Text>{name}</Text>;
};

export const ResourceLink = ({
  type,
  id,
  onClick,
}: {
  type: UsableResource;
  id: string;
  onClick?: () => void;
}) => {
  const Components = ResourceComponents[type];
  const resource = Components.useListItem(id);
  return (
    <Group
      renderRoot={(props) => (
        <Link to={`/${usableResourcePath(type)}/${id}`} {...props} />
      )}
      onClick={(e) => {
        e.stopPropagation();
        onClick?.();
      }}
      style={{ textDecoration: "underline" }}
      c="inherit"
      wrap="nowrap"
    >
      <Components.Icon id={id} />
      <Text style={{ textWrap: "nowrap" }}>{resource?.name ?? "Unknown"}</Text>
      {resource?.template && <TemplateMarker type={type} />}
    </Group>
  );
};

export const ResourceNotFound = ({
  type,
}: {
  type: UsableResource | undefined;
}) => {
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
};
