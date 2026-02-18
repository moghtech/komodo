import { MonacoEditor } from "@/components/monaco";
import { useRead } from "@/lib/hooks";
import Section, { SectionProps } from "@/ui/section";

export interface SwarmSecretInspectSectionProps extends SectionProps {
  swarm: string;
  secret: string;
}

export default function SwarmSecretInspectSection({
  swarm,
  secret,
  ...sectionProps
}: SwarmSecretInspectSectionProps) {
  const {
    data: inspect,
    isPending,
    isError,
  } = useRead("InspectSwarmSecret", {
    swarm,
    secret,
  });

  return (
    <Section
      isPending={isPending}
      error={
        isError
          ? "Failed to inspect swarm secret."
          : !inspect
            ? `No swarm secret found with given id: ${secret}`
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
