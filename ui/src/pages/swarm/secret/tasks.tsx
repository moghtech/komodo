import SwarmTasksSection, {
  SwarmTasksSectionProps,
} from "@/components/swarm/tasks-section";
import { useRead } from "@/lib/hooks";

export interface SwarmSecretTasksSectionProps extends Omit<
  SwarmTasksSectionProps,
  "tasks"
> {
  secret: string | undefined;
}

export default function SwarmSecretTasksSection({
  id,
  secret,
  ...props
}: SwarmSecretTasksSectionProps) {
  const tasks =
    useRead("ListSwarmTasks", { swarm: id }).data?.filter(
      (service) => secret && service.Secrets.includes(secret),
    ) ?? [];
  return <SwarmTasksSection id={id} tasks={tasks} {...props} />;
}
