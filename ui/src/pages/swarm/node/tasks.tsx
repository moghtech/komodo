import SwarmTasksSection, {
  SwarmTasksSectionProps,
} from "@/components/swarm/tasks-section";
import { useRead } from "@/lib/hooks";

export interface SwarmNodeTasksSectionProps extends Omit<
  SwarmTasksSectionProps,
  "tasks"
> {
  nodeId: string | undefined;
}

export default function SwarmNodeTasksSection({
  id,
  nodeId,
  ...props
}: SwarmNodeTasksSectionProps) {
  const tasks =
    useRead("ListSwarmTasks", { swarm: id }).data?.filter(
      (service) => nodeId && service.NodeID === nodeId,
    ) ?? [];
  return <SwarmTasksSection id={id} tasks={tasks} {...props} />;
}
