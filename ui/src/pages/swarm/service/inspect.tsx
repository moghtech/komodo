import { MonacoEditor } from "@/components/monaco";
import { useRead } from "@/lib/hooks";
import Section, { SectionProps } from "@/ui/section";

export interface SwarmServiceInspectSectionProps extends SectionProps {
  swarm: string;
  service: string;
}

export default function SwarmServiceInspectSection({
  swarm,
  service,
  ...sectionProps
}: SwarmServiceInspectSectionProps) {
  const {
    data: inspect,
    isPending,
    isError,
  } = useRead("InspectSwarmService", {
    swarm,
    service,
  });

  return (
    <Section
      isPending={isPending}
      error={
        isError
          ? "Failed to inspect swarm service."
          : !inspect
            ? `No swarm service found with given id: ${service}`
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
