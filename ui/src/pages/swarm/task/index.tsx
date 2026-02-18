import SwarmResourceLink from "@/components/swarm/link";
import { swarmTaskStateIntention } from "@/lib/color";
import { useRead, useSetTitle } from "@/lib/hooks";
import ResourceSubPage from "@/resources/sub-page";
import { useSwarm } from "@/resources/swarm";
import { ICONS } from "@/theme/icons";
import PageGuard from "@/ui/page-guard";
import { useParams } from "react-router-dom";
import SwarmTaskTabs from "./tabs";

export default function SwarmTask() {
  const { id: swarmId, task: __task } = useParams() as {
    id: string;
    task: string;
  };
  const _task = decodeURIComponent(__task);
  const swarm = useSwarm(swarmId);
  const {
    data: tasks,
    isPending,
    isError,
  } = useRead("ListSwarmTasks", {
    swarm: swarmId,
  });
  const task = tasks?.find(
    (task) =>
      _task &&
      // Better to match on start to accept short ids too
      task.ID?.startsWith(_task),
  );
  const node = useRead("ListSwarmNodes", { swarm: swarmId }).data?.find(
    (node) => node.ID === task?.NodeID,
  );
  const service = useRead("ListSwarmServices", { swarm: swarmId }).data?.find(
    (service) => service.ID === task?.ServiceID,
  );
  useSetTitle(`${swarm?.name} | Task | ${task?.ID ?? "Unknown"}`);

  const intent = swarmTaskStateIntention(task?.State, task?.DesiredState);

  return (
    <PageGuard
      isPending={isPending}
      error={
        isError
          ? "Failed to inspect swarm task."
          : !task
            ? `No swarm task found with given id: ${_task}`
            : undefined
      }
    >
      <ResourceSubPage
        entityTypeName="Swarm Task"
        parentType="Swarm"
        parentId={swarmId}
        name={service?.Name}
        icon={ICONS.SwarmTask}
        intent={intent}
        state={task?.State}
        status={`Desired: ${task?.DesiredState}`}
        info={
          <>
            <SwarmResourceLink
              type="Node"
              swarmId={swarmId}
              resourceId={node?.ID}
              name={node?.Hostname}
            />
            <SwarmResourceLink
              type="Service"
              swarmId={swarmId}
              resourceId={service?.ID}
              name={service?.Name}
            />
            {task?.Configs.map((config) => (
              <SwarmResourceLink
                key={config}
                type="Config"
                swarmId={swarmId}
                resourceId={config}
                name={config}
              />
            ))}
            {task?.Secrets.map((secret) => (
              <SwarmResourceLink
                key={secret}
                type="Secret"
                swarmId={swarmId}
                resourceId={secret}
                name={secret}
              />
            ))}
          </>
        }
      >
        {swarm && <SwarmTaskTabs swarm={swarm} task={_task} intent={intent} />}
      </ResourceSubPage>
    </PageGuard>
  );
}
