import { fmtResourceType } from "@/lib/formatting";
import { UsableResource } from "@/resources";
import { Badge, HoverCard, Text } from "@mantine/core";

export const TemplateMarker = ({ type }: { type: UsableResource }) => {
  return (
    <HoverCard>
      <HoverCard.Target>
        <Badge radius="sm" px="0.3rem" color="gray" c="inherit">
          T
        </Badge>
      </HoverCard.Target>
      <HoverCard.Dropdown>
        <Text>This {fmtResourceType(type).toLowerCase()} is a template.</Text>
      </HoverCard.Dropdown>
    </HoverCard>
  );
};
