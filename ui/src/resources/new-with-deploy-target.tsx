import { useRead } from "@/lib/hooks";
import { useState } from "react";
import NewResource from "./new";
import { Types } from "komodo_client";
import ResourceSelector from "./selector";
import { Divider } from "@mantine/core";

/** Used by Stacks and Deployments */
export default function NewResourceWithDeployTarget({
  type,
  serverId: _serverId,
  swarmId: _swarmId,
}: {
  type: "Stack" | "Deployment";
  serverId?: string;
  swarmId?: string;
}) {
  const [serverId, setServerId] = useState("");
  const [swarmId, setSwarmId] = useState("");
  const showSwarms = !!useRead("ListSwarms", {}).data?.length;
  return (
    <NewResource<Types.SwarmConfig | Types.DeploymentConfig>
      type={type}
      config={() => ({
        server_id: _serverId ?? serverId,
        swarm_id: _swarmId ?? swarmId,
      })}
      extraInputs={
        !(_serverId ?? _swarmId) ? (
          <>
            {!swarmId && (
              <ResourceSelector
                type="Server"
                selected={serverId}
                onSelect={setServerId}
                targetProps={{ w: "100%", maw: "100%" }}
                width="target"
                position="bottom"
                clearable
              />
            )}
            {showSwarms && !serverId && (
              <>
                {!swarmId && <Divider label="OR" my="-10" />}
                <ResourceSelector
                  type="Swarm"
                  selected={swarmId}
                  onSelect={setSwarmId}
                  targetProps={{ w: "100%", maw: "100%" }}
                  width="target"
                  position="bottom"
                  clearable
                />
              </>
            )}
          </>
        ) : undefined
      }
      showTemplateSelector={
        !!_serverId || !!_swarmId || (!serverId && !swarmId)
      }
    />
  );
}
