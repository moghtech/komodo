import { useRead, useSetTitle } from "@/lib/hooks";
import { useSwarm } from "@/resources/swarm";
import { ICONS } from "@/theme/icons";
import { Badge } from "@mantine/core";
import { useParams } from "react-router-dom";
import RemoveSwarmResource from "@/components/swarm/remove";
import ResourceSubPage from "@/resources/sub-page";
import SwarmConfigTabs from "./tabs";
import PageGuard from "@/ui/page-guard";

export default function SwarmConfig() {
  const { id: swarmId, config: __config } = useParams() as {
    id: string;
    config: string;
  };
  const _config = decodeURIComponent(__config);
  const swarm = useSwarm(swarmId);
  const inUse = useRead("ListSwarmConfigs", { swarm: swarmId }).data?.find(
    (config) => config.ID === _config || config.Name === _config,
  )?.InUse;
  const {
    data: config,
    isPending,
    isError,
  } = useRead("InspectSwarmConfig", {
    swarm: swarmId,
    config: _config,
  });
  useSetTitle(`${swarm?.name} | Config | ${config?.Spec?.Name ?? "Unknown"}`);

  const intent = inUse ? "Good" : "Critical";

  return (
    <PageGuard
      isPending={isPending}
      error={
        isError
          ? "Failed to inspect swarm config."
          : !config
            ? `No swarm config found with given name: ${_config}`
            : undefined
      }
    >
      <ResourceSubPage
        entityTypeName="Swarm Config"
        parentType="Swarm"
        parentId={swarmId}
        name={config?.Spec?.Name}
        icon={ICONS.SwarmConfig}
        intent={intent}
        state={
          !inUse && (
            <Badge variant="filled" color="red">
              Unused
            </Badge>
          )
        }
        executions={
          config?.ID && (
            <RemoveSwarmResource
              swarmId={swarmId}
              type="Config"
              resourceId={config.ID}
              resourceName={config.Spec?.Name}
              disabled={inUse}
            />
          )
        }
      >
        {swarm && (
          <SwarmConfigTabs swarm={swarm} config={_config} intent={intent} />
        )}
      </ResourceSubPage>
    </PageGuard>
  );
}
