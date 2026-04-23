import { Badge, Group, GroupProps } from "@mantine/core";

export interface LabelsGroupProps extends GroupProps {
  labels: [string, string][];
  showEllipsis?: boolean;
}

export default function LabelsGroup({
  labels,
  showEllipsis,
  ...groupProps
}: LabelsGroupProps) {
  return (
    <Group gap="xs" {...groupProps}>
      {labels.map(([key, val]) => (
        <Badge key={key} tt="none" fw="normal" fz={{ base: "sm", sm: "md" }}>
          {key}=<b>{val}</b>
        </Badge>
      ))}
      {showEllipsis && (
        <Badge tt="none" fw="normal" fz={{ base: "sm", sm: "md" }}>
          ...
        </Badge>
      )}
    </Group>
  );
}
