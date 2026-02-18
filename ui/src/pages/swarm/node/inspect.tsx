import { MonacoEditor } from "@/components/monaco";
import { useRead } from "@/lib/hooks";
import Section, { SectionProps } from "@/ui/section";
import { Center, Loader, Text } from "@mantine/core";

export interface SwarmNodeInspectSectionProps extends SectionProps {
  swarm: string;
  node: string;
}

export default function SwarmNodeInspectSection({
  swarm,
  node,
  ...sectionProps
}: SwarmNodeInspectSectionProps) {
  const {
    data: inspect,
    isPending,
    isError,
  } = useRead("InspectSwarmNode", {
    swarm,
    node,
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

  if (!node) {
    return (
      <Center h="30vh">
        <Text>No swarm node found with given id: {node}</Text>
      </Center>
    );
  }

  return (
    <Section
      isPending={isPending}
      error={
        isError
          ? "Failed to inspect swarm node."
          : !inspect
            ? `No swarm node found with given id: ${node}`
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
