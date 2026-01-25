import { ResourceComponents, UsableResource } from ".";

export const ResourceNameSimple = ({
  type,
  id,
}: {
  type: UsableResource;
  id: string;
}) => {
  const Components = ResourceComponents[type];
  const name = Components.useListItem(id)?.name ?? "unknown";
  return <div>{name}</div>;
};
