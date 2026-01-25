import { fmtResourceType } from "@/lib/formatting";
import { UsableResource } from "@/resources";
import { Badge, Text, Tooltip } from "@mantine/core";

export const TemplateMarker = ({ type }: { type: UsableResource }) => {
  return (
    <Tooltip
      label={
        <Text>This {fmtResourceType(type).toLowerCase()} is a template.</Text>
      }
    >
      <Badge>T</Badge>
    </Tooltip>
  );
};
