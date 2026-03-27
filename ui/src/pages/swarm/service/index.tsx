import SwarmResourceLink from "@/components/swarm/link";
import RemoveSwarmResource from "@/components/swarm/remove";
import { swarmStateIntention } from "@/lib/color";
import { useRead, useSetTitle } from "@/lib/hooks";
import ResourceSubPage from "@/resources/sub-page";
import { useSwarm } from "@/resources/swarm";
import { ICONS } from "@/theme/icons";
import PageGuard from "@/ui/page-guard";
import { useParams } from "react-router-dom";
import SwarmServiceTabs from "./tabs";

export default function SwarmService() {
  const { id: swarmId, service: __service } = useParams() as {
    id: string;
    service: string;
  };
  const _service = decodeURIComponent(__service);
  const swarm = useSwarm(swarmId);
  const {
    data: services,
    isPending,
    isError,
  } = useRead("ListSwarmServices", {
    swarm: swarmId,
  });
  const service = services?.find(
    (service) =>
      _service &&
      // First match on name here.
      // Then better to match on ID start to accept short ids too.
      (service.Name === _service || service.ID?.startsWith(_service)),
  );
  const tasks =
    useRead("ListSwarmTasks", {
      swarm: swarmId,
    }).data?.filter((task) => service?.ID && task.ServiceID === service.ID) ??
    [];
  useSetTitle(
    `${swarm?.name} | Service | ${service?.Name ?? service?.ID ?? "Unknown"}`,
  );

  const state = service?.State;
  const intent = swarmStateIntention(state);

  return (
    <PageGuard
      isPending={isPending}
      error={
        isError
          ? "Failed to inspect swarm service."
          : !service
            ? `No swarm service found with given name: ${_service}`
            : undefined
      }
    >
      <ResourceSubPage
        entityTypeName="Swarm Service"
        parentType="Swarm"
        parentId={swarmId}
        name={service?.Name}
        icon={ICONS.SwarmService}
        intent={intent}
        state={state}
        status={`${tasks.length} Task${tasks.length === 1 ? "" : "s"}`}
        info={
          <>
            {service?.Configs.map((config) => (
              <SwarmResourceLink
                key={config}
                type="Config"
                swarmId={swarmId}
                resourceId={config}
                name={config}
              />
            ))}
            {service?.Secrets.map((secret) => (
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
        executions={
          service?.Name && (
            <RemoveSwarmResource
              swarmId={swarmId}
              type="Service"
              resourceId={service?.Name}
            />
          )
        }
      >
        {swarm && service && (
          <SwarmServiceTabs
            swarm={swarm}
            serviceId={service.ID}
            service={_service}
            intent={intent}
          />
        )}
      </ResourceSubPage>
    </PageGuard>
  );
}
