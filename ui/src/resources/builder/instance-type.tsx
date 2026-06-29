import { Bot } from "lucide-react";
import { useBuilder } from ".";
import ResourceLink from "../link";
import { Group } from "@mantine/core";

export default function BuilderInstanceType({ id }: { id: string }) {
  let info = useBuilder(id)?.info;
  if (info?.builder_type === "Server") {
    return info.instance_type
      ?.split(",")
      .map((id) => <ResourceLink type="Server" id={id.trim()} />);
  } else {
    return (
      <Group gap="xs">
        <Bot className="w-4 h-4" />
        {info?.instance_type}
      </Group>
    );
  }
}
