import { usableResourcePath } from "@/lib/utils";
import { ResourceComponents, UsableResource } from ".";
import { Group, Text } from "@mantine/core";
import { Link } from "react-router-dom";
import { TemplateMarker } from "@/components/template-marker";

export default function ResourceLink({
  type,
  id,
  onClick,
}: {
  type: UsableResource;
  id: string;
  onClick?: () => void;
}) {
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
      wrap="nowrap"
      gap="xs"
    >
      <Components.Icon id={id} />
      <Text className="hover-underline" style={{ textWrap: "nowrap" }}>
        {resource?.name ?? "Unknown"}
      </Text>
      {resource?.template && <TemplateMarker type={type} />}
    </Group>
  );
}
