import { UsableResource } from "@/resources";
import { Group } from "@mantine/core";
import { Types } from "komodo_client";
import TemplateSwitch from "@/components/template-switch";
import DeleteResource from "@/resources/delete";

export default function ResourceHeaderAction({
  type,
  id,
  resource,
}: {
  type: UsableResource;
  id: string;
  resource: Types.ResourceListItem<unknown> | undefined;
}) {
  return (
    <Group>
      <TemplateSwitch type={type} id={id} resource={resource} />
      <DeleteResource type={type} id={id} />
    </Group>
  );
}
