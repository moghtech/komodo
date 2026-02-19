import EntityHeader, { EntityHeaderProps } from "@/ui/entity-header";
import { UsableResource } from ".";
import { useWrite } from "@/lib/hooks";
import { notifications } from "@mantine/notifications";
import ResourceHeaderAction from "./header-action";
import { Types } from "komodo_client";

export interface ResourceHeaderProps extends Omit<EntityHeaderProps, "action"> {
  type: UsableResource;
  id: string;
  resource: Types.ResourceListItem<unknown> | undefined;
}

export default function ResourceHeader({
  type,
  id,
  resource,
  ...props
}: ResourceHeaderProps) {
  const { mutateAsync: rename, isPending: renamePending } = useWrite(
    `Rename${type}`,
    {
      onSuccess: () => {
        notifications.show({ message: "Renamed " + type, color: "green" });
      },
    },
  );
  return (
    <EntityHeader
      action={<ResourceHeaderAction type={type} id={id} resource={resource} />}
      onRename={(name) => rename({ id, name })}
      renamePending={renamePending}
      {...props}
    />
  );
}
