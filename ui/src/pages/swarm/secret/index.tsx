import RemoveSwarmResource from "@/components/swarm/remove";
import { useRead, useSetTitle } from "@/lib/hooks";
import ResourceSubPage from "@/resources/sub-page";
import { useSwarm } from "@/resources/swarm";
import { ICONS } from "@/theme/icons";
import PageGuard from "@/ui/page-guard";
import { Badge } from "@mantine/core";
import { useParams } from "react-router-dom";
import SwarmSecretTabs from "./tabs";

export default function SwarmSecret() {
  const { id: swarmId, secret: __secret } = useParams() as {
    id: string;
    secret: string;
  };
  const _secret = decodeURIComponent(__secret);
  const swarm = useSwarm(swarmId);
  const inUse = useRead("ListSwarmSecrets", { swarm: swarmId }).data?.find(
    (secret) => secret.ID === _secret || secret.Name === _secret,
  )?.InUse;
  const {
    data: secret,
    isPending,
    isError,
  } = useRead("InspectSwarmSecret", {
    swarm: swarmId,
    secret: _secret,
  });
  useSetTitle(`${swarm?.name} | Secret | ${secret?.Spec?.Name ?? "Unknown"}`);

  const intent = inUse ? "Good" : "Critical";

  return (
    <PageGuard
      isPending={isPending}
      error={
        isError
          ? "Failed to inspect swarm secret."
          : !secret
            ? `No swarm secret found with given name: ${_secret}`
            : undefined
      }
    >
      <ResourceSubPage
        entityTypeName="Swarm Config"
        parentType="Swarm"
        parentId={swarmId}
        name={secret?.Spec?.Name}
        icon={ICONS.SwarmSecret}
        intent={intent}
        state={
          !inUse && (
            <Badge variant="filled" color="blue">
              Unused
            </Badge>
          )
        }
        executions={
          secret?.ID && (
            <RemoveSwarmResource
              swarmId={swarmId}
              type="Secret"
              resourceId={secret.ID}
              resourceName={secret.Spec?.Name}
              disabled={inUse}
            />
          )
        }
      >
        {swarm && (
          <SwarmSecretTabs swarm={swarm} secret={_secret} intent={intent} />
        )}
      </ResourceSubPage>
    </PageGuard>
  );
}
