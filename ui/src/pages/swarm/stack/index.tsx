import RemoveSwarmResource from "@/components/swarm/remove";
import { swarmStateIntention } from "@/lib/color";
import { useRead, useSetTitle } from "@/lib/hooks";
import ResourceSubPage from "@/resources/sub-page";
import { useSwarm } from "@/resources/swarm";
import { ICONS } from "@/theme/icons";
import PageGuard from "@/ui/page-guard";
import { useParams } from "react-router-dom";
import SwarmStackTabs from "./tabs";

export default function SwarmStack() {
  const { id: swarmId, stack: __stack } = useParams() as {
    id: string;
    stack: string;
  };
  const _stack = decodeURIComponent(__stack);
  const swarm = useSwarm(swarmId);
  const {
    data: stack,
    isPending,
    isError,
  } = useRead("InspectSwarmStack", {
    swarm: swarmId,
    stack: _stack,
  });
  useSetTitle(`${swarm?.name} | Stack | ${stack?.Name ?? "Unknown"}`);

  const state = stack?.State;
  const intent = swarmStateIntention(state);

  return (
    <PageGuard
      isPending={isPending}
      error={
        isError
          ? "Failed to inspect swarm stack."
          : !stack
            ? `No swarm stack found with given name: ${_stack}`
            : undefined
      }
    >
      <ResourceSubPage
        entityTypeName="Swarm Stack"
        parentType="Swarm"
        parentId={swarmId}
        name={stack?.Name}
        icon={ICONS.SwarmStack}
        intent={intent}
        state={state}
        status={`${stack?.Services.length} Service${stack?.Services.length === 1 ? "" : "s"}`}
        executions={
          stack?.Name && (
            <RemoveSwarmResource
              swarmId={swarmId}
              type="Stack"
              resourceId={stack?.Name}
            />
          )
        }
      >
        {swarm && stack && (
          <SwarmStackTabs
            swarm={swarm}
            stack={stack}
            intent={intent}
          />
        )}
      </ResourceSubPage>
    </PageGuard>
  );
}
