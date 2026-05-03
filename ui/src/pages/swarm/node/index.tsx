import { useParams } from "react-router-dom";
import { swarmNodeStateIntention } from "@/lib/color";
import { useRead, useSetTitle } from "@/lib/hooks";
import { useSwarm } from "@/resources/swarm";
import { ICONS } from "@/lib/icons";
import RemoveSwarmResource from "@/components/swarm/remove";
import ResourceSubPage from "@/resources/sub-page";
import SwarmNodeTabs from "./tabs";
import { PageGuard } from "mogh_ui";
import UpdateSwarmNodes from "@/resources/swarm/docker/nodes/update";

export default function SwarmNode() {
  const { id: swarmId, node: __node } = useParams() as {
    id: string;
    node: string;
  };
  const _node = decodeURIComponent(__node);
  const swarm = useSwarm(swarmId);
  const {
    data: node,
    isPending,
    isError,
  } = useRead("InspectSwarmNode", {
    swarm: swarmId,
    node: _node,
  });
  const state = node?.Status?.State;
  useSetTitle(
    `${swarm?.name} | Node | ${node?.Description?.Hostname ?? "Unknown"}`,
  );

  const intent = swarmNodeStateIntention(state);

  return (
    <PageGuard
      isPending={isPending}
      error={
        isError
          ? "Failed to inspect swarm node."
          : !node
            ? `No swarm node found with given id: ${_node}`
            : undefined
      }
    >
      <ResourceSubPage
        entityTypeName="Swarm Node"
        parentType="Swarm"
        parentId={swarmId}
        name={node?.Description?.Hostname}
        icon={ICONS.SwarmNode}
        intent={intent}
        state={state}
        status={node?.Spec?.Role}
        executions={
          node?.ID && (
            <>
              <UpdateSwarmNodes
                swarm={swarmId}
                nodes={
                  node.Description?.Hostname ? [node.Description?.Hostname] : []
                }
              />
              <RemoveSwarmResource
                swarmId={swarmId}
                type="Node"
                resourceId={node.ID}
                resourceName={node.Description?.Hostname}
              />
            </>
          )
        }
      >
        {swarm && node && (
          <SwarmNodeTabs
            swarm={swarm}
            _node={_node}
            node={node}
            intent={intent}
          />
        )}
      </ResourceSubPage>
    </PageGuard>
  );
}
