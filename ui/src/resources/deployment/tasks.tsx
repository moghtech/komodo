import { useRead } from "@/lib/hooks";
import { useDeployment } from ".";
import SwarmServiceTasksSection, {
  SwarmServiceTasksSectionProps,
} from "@/pages/swarm/service/tasks";
import { useState } from "react";

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
  ).data?.find((service) => service.Name === deployment?.name);
  const _search = useState("");

  if (!swarmId || !service) return;

  return (
    <SwarmServiceTasksSection
      id={swarmId}
      serviceId={service.ID}
      _search={_search}
      {...props}
    />
  );
}
