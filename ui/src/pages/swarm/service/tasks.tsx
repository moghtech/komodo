import SwarmTasksSection, {
  SwarmTasksSectionProps,
} from "@/components/swarm/tasks-section";
import { useRead } from "@/lib/hooks";

export interface SwarmServiceTasksSectionProps extends Omit<
  SwarmTasksSectionProps,
  "tasks"
> {
  serviceId: string | undefined;
}

export default function SwarmServiceTasksSection({
  id,
  serviceId,
  ...props
}: SwarmServiceTasksSectionProps) {
  const tasks =
    useRead(
      "ListSwarmTasks",
      { swarm: id },
      { enabled: !!serviceId },
    ).data?.filter((task) => serviceId && task.ServiceID === serviceId) ?? [];
  return <SwarmTasksSection id={id} tasks={tasks} {...props} />;
}
