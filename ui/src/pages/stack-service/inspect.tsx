import { MonacoEditor } from "mogh_ui";
import { usePermissions, useRead } from "@/lib/hooks";
import { Section } from "mogh_ui";
import { Center, Text } from "@mantine/core";
import { Types } from "komodo_client";
import { ReactNode } from "react";

export interface StackServiceInspectProps {
  stackId: string;
  service: string;
  useSwarm: boolean;
  titleOther: ReactNode;
}

export default function StackServiceInspect({
  stackId,
  service,
  useSwarm,
  titleOther,
}: StackServiceInspectProps) {
  const { specific } = usePermissions({ type: "Stack", id: stackId });
  const canInspect = specific.includes(Types.SpecificPermission.Inspect);

  const { data: inspect } = useRead(
    `InspectStack${useSwarm ? "SwarmService" : "Container"}`,
    {
      stack: stackId,
      service,
    },
    { enabled: canInspect },
  );

  if (!canInspect) {
    return (
      <Section titleOther={titleOther}>
        <Center className="min-h-[60vh]">
          <Text>
            User does not have permission to inspect this Stack service.
          </Text>
        </Center>
      </Section>
    );
  }

  return (
    <Section titleOther={titleOther}>
      <MonacoEditor
        value={inspect ? JSON.stringify(inspect, null, 2) : "NO DATA"}
        language="json"
        readOnly
      />
    </Section>
  );
}
