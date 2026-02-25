import { MonacoEditor } from "@/components/monaco";
import { useRead } from "@/lib/hooks";
import Section, { SectionProps } from "@/ui/section";

export interface SwarmStackInspectSectionProps extends SectionProps {
  swarm: string;
  stack: string;
}

export default function SwarmStackInspectSection({
  swarm,
  stack,
  ...sectionProps
}: SwarmStackInspectSectionProps) {
  const {
    data: inspect,
    isPending,
    isError,
  } = useRead("InspectSwarmStack", {
    swarm,
    stack,
  });

  return (
    <Section
      isPending={isPending}
      error={
        isError
          ? "Failed to inspect swarm stack."
          : !inspect
            ? `No swarm stack found with given name: ${stack}`
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
