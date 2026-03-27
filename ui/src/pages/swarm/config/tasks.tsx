import SwarmTasksSection, {
  SwarmTasksSectionProps,
} from "@/components/swarm/tasks-section";
import { useRead } from "@/lib/hooks";

export interface SwarmConfigTasksSectionProps extends Omit<
  SwarmTasksSectionProps,
  "tasks"
> {
  config: string | undefined;
}

export default function SwarmConfigTasksSection({
  id,
  config,
  ...props
}: SwarmConfigTasksSectionProps) {
  const tasks =
    useRead("ListSwarmTasks", { swarm: id }).data?.filter(
      (service) => config && service.Configs.includes(config),
    ) ?? [];
  return <SwarmTasksSection id={id} tasks={tasks} {...props} />;
}
