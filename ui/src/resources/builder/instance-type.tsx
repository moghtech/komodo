import { Bot } from "lucide-react";
import { useBuilder } from ".";
import ResourceLink from "../link";
import { Group } from "@mantine/core";
import { DividedChildren } from "mogh_ui";

export default function BuilderInstanceType({ id }: { id: string }) {
  let info = useBuilder(id)?.info;
  if (info?.builder_type === "Server") {
    return (
      <DividedChildren wrap="nowrap" gap="xs">
        {info.instance_type?.split(",").map((id) => (
          <ResourceLink key={id.trim()} type="Server" id={id.trim()} />
        ))}
      </DividedChildren>
    );
  } else {
    return (
      <Group gap="xs">
        <Bot className="w-4 h-4" />
        {info?.instance_type}
      </Group>
    );
  }
}
