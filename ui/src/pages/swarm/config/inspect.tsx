import { MonacoEditor } from "@/components/monaco";
import { useRead } from "@/lib/hooks";
import Section, { SectionProps } from "@/ui/section";
import { Center, Loader, Text } from "@mantine/core";

export interface SwarmConfigInspectSectionProps extends SectionProps {
  swarm: string;
  config: string;
}

export default function SwarmConfigInspectSection({
  swarm,
  config,
  ...sectionProps
}: SwarmConfigInspectSectionProps) {
  const {
    data: inspect,
    isPending,
    isError,
  } = useRead("InspectSwarmConfig", {
    swarm,
    config,
  });

  if (isPending) {
    return (
      <Center h="30vh">
        <Loader size="xl" />
      </Center>
    );
  }

  if (isError) {
    return (
      <Center h="30vh">
        <Text>Failed to inspect swarm node.</Text>
      </Center>
    );
  }

  if (!config) {
    return (
      <Center h="30vh">
        <Text>No swarm config found with given id: {config}</Text>
      </Center>
    );
  }

  return (
    <Section {...sectionProps}>
      <MonacoEditor
        value={JSON.stringify(inspect, null, 2)}
        language="json"
        readOnly
      />
    </Section>
  );
}
