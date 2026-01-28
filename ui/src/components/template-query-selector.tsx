import { useTemplatesQueryBehavior } from "@/lib/hooks";
import { Badge, Select } from "@mantine/core";
import { Types } from "komodo_client";

const BEHAVIORS = Object.keys(Types.TemplatesQueryBehavior);

export default function TemplateQuerySelector() {
  const [value, set] = useTemplatesQueryBehavior();
  return (
    <Select
      w="200"
      leftSection={<Badge radius="sm" px="0.3rem" color="accent" c="inherit">T</Badge>}
      value={value}
      data={BEHAVIORS.map((value) => ({ value, label: value + " Templates" }))}
      onChange={(value) =>
        set(
          (value as Types.TemplatesQueryBehavior) ??
            Types.TemplatesQueryBehavior.Exclude,
        )
      }
    />
  );
}
