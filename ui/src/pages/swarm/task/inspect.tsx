import { MonacoEditor } from "mogh_ui";
import { useRead } from "@/lib/hooks";
import { Section, SectionProps } from "mogh_ui";

export interface SwarmTaskInspectSectionProps extends SectionProps {
  swarm: string;
  task: string;
}

export default function SwarmTaskInspectSection({
  swarm,
  task,
  ...sectionProps
}: SwarmTaskInspectSectionProps) {
  const {
    data: inspect,
    isPending,
    isError,
  } = useRead("InspectSwarmTask", {
    swarm,
    task,
  });

  return (
    <Section
      isPending={isPending}
      error={
        isError
          ? "Failed to inspect swarm task."
          : !inspect
            ? `No swarm task found with given id: ${task}`
            : undefined
      }
      {...sectionProps}
    >
      <MonacoEditor
        value={JSON.stringify(inspect, null, 2)}
        language="json"
        readOnly
      />
    </Section>
  );
}
