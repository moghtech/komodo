import { useRead } from "@/lib/hooks";
import { deploymentContainerName, useDeployment } from ".";
import SwarmServiceTasksSection, {
  SwarmServiceTasksSectionProps,
} from "@/pages/swarm/service/tasks";
import { useState } from "react";
import { Section } from "mogh_ui";
import { Text } from "@mantine/core";

export interface DeploymentTasksSectionProps extends Omit<
  Omit<SwarmServiceTasksSectionProps, "id">,
  "serviceId"
> {
  deploymentId: string | undefined;
}

export default function DeploymentTasksSection({
  deploymentId,
  ...props
}: DeploymentTasksSectionProps) {
  const deployment = useDeployment(deploymentId);
  const swarmId = deployment?.info.swarm_id;
  const service = useRead(
    "ListSwarmServices",
    { swarm: swarmId! },
    { enabled: !!swarmId },
  ).data?.find(
    (service) => service.Name === deploymentContainerName(deployment),
  );
  const _search = useState("");

  if (!swarmId || !service) {
    return (
      <Section {...props}>
        <Text>Did not find {!swarmId ? "Swarm" : "Swarm Service"}</Text>
      </Section>
    );
  }

  return (
    <SwarmServiceTasksSection
      id={swarmId}
      serviceId={service.ID}
      _search={_search}
      {...props}
    />
  );
}
