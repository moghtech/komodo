import { useInvalidate, usePermissions, useWrite } from "@/lib/hooks";
import { UsableResource } from "@/resources";
import LabelledSwitch from "@/ui/labelled-switch";
import { notifications } from "@mantine/notifications";
import { Types } from "komodo_client";

export default function TemplateSwitch({
  type,
  id,
  resource,
}: {
  type: UsableResource;
  id: string;
  resource: Types.ResourceListItem<unknown> | undefined;
}) {
  const inv = useInvalidate();
  const { canWrite } = usePermissions({ type, id });
  const { mutate, isPending } = useWrite("UpdateResourceMeta", {
    onSuccess: () => {
      inv([`List${type}s`], [`Get${type}`]);
      notifications.show({
        message: `Updated is template on ${type} ${resource?.name}`,
        color: "green",
      });
    },
  });
  return (
    <LabelledSwitch
      label="Template"
      checked={resource?.template}
      onCheckedChange={(template) =>
        canWrite &&
        resource &&
        !isPending &&
        mutate({ target: { type, id }, template })
      }
    />
  );
}
