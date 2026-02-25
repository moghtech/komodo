import { usableResourcePath } from "@/lib/utils";
import { ResourceComponents, UsableResource } from ".";
import { Group, GroupProps, Text, TextProps } from "@mantine/core";
import { Link } from "react-router-dom";
import { TemplateMarker } from "@/components/template-marker";

export interface ResourceLinkProps extends TextProps {
  type: UsableResource;
  id: string;
  onClick?: () => void;
  noColor?: boolean;
  groupProps?: GroupProps;
  iconSize?: string | number;
}

export default function ResourceLink({
  type,
  id,
  onClick,
  noColor,
  groupProps,
  iconSize,
  ...textProps
}: ResourceLinkProps) {
  const RC = ResourceComponents[type];
  const resource = RC.useListItem(id);
  return (
    <Group
      title={`${type} - ${resource?.name}`}
      renderRoot={(props) => (
        <Link to={`/${usableResourcePath(type)}/${id}`} {...props} />
      )}
      onClick={(e) => {
        e.stopPropagation();
        onClick?.();
      }}
      wrap="nowrap"
      gap="xs"
      {...groupProps}
    >
      <RC.Icon id={id} noColor={noColor} size={iconSize} />
      <Text
        className="hover-underline"
        style={{ textWrap: "nowrap" }}
        fz={{ base: "sm", lg: "md" }}
        {...textProps}
      >
        {resource?.name ?? "Unknown"}
      </Text>
      {resource?.template && <TemplateMarker type={type} />}
    </Group>
  );
}
