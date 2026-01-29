import { useTemplatesQueryBehavior } from "@/lib/hooks";
import { Badge, Center, Select } from "@mantine/core";
import { Types } from "komodo_client";
import { Ban } from "lucide-react";

const BEHAVIORS = Object.keys(Types.TemplatesQueryBehavior);

export default function TemplateQuerySelector() {
  const [value, set] = useTemplatesQueryBehavior();
  const isExclude = value === Types.TemplatesQueryBehavior.Exclude;
  return (
    <Select
      w="200"
      leftSection={
        <Badge
          radius="sm"
          px={isExclude ? "0.2rem" : "0.3rem"}
          py={isExclude ? "0.3rem" : "0.1rem"}
          color="accent"
          c="dimmed"
          h="fit-content"
        >
          <Center>{isExclude ? <Ban size="0.7rem" /> : "T"}</Center>
        </Badge>
      }
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
