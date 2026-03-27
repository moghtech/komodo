import { Text, TextProps } from "@mantine/core";
import { ResourceComponents, UsableResource } from ".";

export interface ResourceNameProps extends TextProps {
  type: UsableResource;
  id: string;
}

export default function ResourceName({
  type,
  id,
  ...props
}: ResourceNameProps) {
  const Components = ResourceComponents[type];
  const name = Components.useListItem(id)?.name ?? "unknown";
  return <Text {...props}>{name}</Text>;
}
