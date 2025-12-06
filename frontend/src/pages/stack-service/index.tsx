import { Section } from "@components/layouts";
import {
  ResourceDescription,
  ResourceLink,
  ResourcePageHeader,
} from "@components/resources/common";
import { useStack } from "@components/resources/stack";
import {
  DeployStack,
  DestroyStack,
  PauseUnpauseStack,
  PullStack,
  RestartStack,
  StartStopStack,
} from "@components/resources/stack/actions";
import {
  container_state_intention,
  stroke_color_class_by_intention,
  swarm_state_intention,
} from "@lib/color";
import {
  usePermissions,
  useRead,
  useSetTitle,
  useContainerPortsMap,
} from "@lib/hooks";
import { cn } from "@lib/utils";
import { Types } from "komodo_client";
import { ChevronLeft, Layers2, Zap } from "lucide-react";
import { Link, useParams } from "react-router-dom";
import { Button } from "@ui/button";
import { ExportButton } from "@components/export";
import { ContainerPortLink, DockerResourceLink } from "@components/util";
import { ResourceNotifications } from "@pages/resource-notifications";
import { Fragment } from "react/jsx-runtime";
import { StackServiceTabs } from "./tabs";
import { SwarmLink } from "@components/resources/swarm";

type IdServiceComponent = React.FC<{ id: string; service?: string }>;

const Actions: { [action: string]: IdServiceComponent } = {
  DeployStack,
  PullStack,
  RestartStack,
  PauseUnpauseStack,
  StartStopStack,
  DestroyStack,
};

export default function StackServicePage() {
  const { type, id, service } = useParams() as {
    type: string;
    id: string;
    service: string;
  };
  if (type !== "stacks") {
    return <div>This resource type does not have any services.</div>;
  }
  return <StackServicePageInner stack_id={id} service={service} />;
}

const StackServicePageInner = ({
  stack_id,
  service: _service,
}: {
  stack_id: string;
  service: string;
}) => {
  const stack = useStack(stack_id);
  useSetTitle(`${stack?.name} | ${_service}`);
  const { canExecute, canWrite } = usePermissions({
    type: "Stack",
    id: stack_id,
  });
  const services = useRead("ListStackServices", { stack: stack_id }).data;
  const service = services?.find((s) => s.service === _service);
  const container = service?.container;
  const swarm_service = service?.swarm_service;
  const ports_map = useContainerPortsMap(container?.ports ?? []);
  const state = swarm_service?.State
    ? swarm_service?.State
    : (container?.state ?? Types.ContainerStateStatusEnum.Empty);
  const intention = swarm_service?.State
    ? swarm_state_intention(swarm_service.State)
    : container_state_intention(
        container?.state ?? Types.ContainerStateStatusEnum.Empty
      );
  const stroke_color = stroke_color_class_by_intention(intention);

  return (
    <div>
      <div className="w-full flex items-center justify-between mb-12">
        <Link to={"/stacks/" + stack_id}>
          <Button className="gap-2" variant="secondary">
            <ChevronLeft className="w-4" />
            Back
          </Button>
        </Link>
        <div className="flex items-center gap-4">
          <ExportButton targets={[{ type: "Stack", id: stack_id }]} />
        </div>
      </div>
      <div className="flex flex-col xl:flex-row gap-4">
        {/* HEADER */}
        <div className="w-full flex flex-col gap-4">
          <div className="flex flex-col gap-2 border rounded-md">
            {/* <Components.ResourcePageHeader id={id} /> */}
            <ResourcePageHeader
              type={undefined}
              id={undefined}
              intent={intention}
              icon={<Layers2 className={cn("w-8 h-8", stroke_color)} />}
              resource={undefined}
              name={_service}
              state={state}
              status={
                swarm_service
                  ? `${swarm_service.Replicas} Replica${swarm_service.Replicas === 1 ? "" : "s"}`
                  : container?.status
              }
            />
            <div className="flex flex-col pb-2 px-4">
              <div className="flex items-center gap-x-4 gap-y-0 flex-wrap text-muted-foreground">
                <ResourceLink type="Stack" id={stack_id} />
                {/* SWARM ONLY */}
                {stack?.info.swarm_id && (
                  <>
                    |
                    <ResourceLink type="Swarm" id={stack.info.swarm_id} />
                    {swarm_service?.Name && (
                      <>
                        |
                        <SwarmLink
                          type="Service"
                          swarm_id={stack.info.swarm_id}
                          resource_id={swarm_service.ID}
                          name={swarm_service.Name}
                        />
                      </>
                    )}
                  </>
                )}
                {/* SERVER ONLY */}
                {!stack?.info.swarm_id && stack?.info.server_id && (
                  <>
                    |
                    <ResourceLink type="Server" id={stack.info.server_id} />
                    {container?.name && (
                      <>
                        |
                        <DockerResourceLink
                          type="container"
                          server_id={stack.info.server_id}
                          name={container.name}
                          muted
                        />
                      </>
                    )}
                    {container?.image && (
                      <>
                        |
                        <DockerResourceLink
                          type="image"
                          server_id={stack.info.server_id}
                          name={container.image}
                          id={container.image_id}
                          muted
                        />
                      </>
                    )}
                    {container?.networks?.map((network) => (
                      <Fragment key={network}>
                        |
                        <DockerResourceLink
                          type="network"
                          server_id={stack.info.server_id}
                          name={network}
                          muted
                        />
                      </Fragment>
                    ))}
                    {container?.volumes?.map((volume) => (
                      <Fragment key={volume}>
                        |
                        <DockerResourceLink
                          type="volume"
                          server_id={stack.info.server_id}
                          name={volume}
                          muted
                        />
                      </Fragment>
                    ))}
                    {Object.keys(ports_map).map((host_port) => (
                      <Fragment key={host_port}>
                        |
                        <ContainerPortLink
                          host_port={host_port}
                          ports={ports_map[host_port]}
                          server_id={stack.info.server_id}
                        />
                      </Fragment>
                    ))}
                  </>
                )}
              </div>
            </div>
          </div>
          <ResourceDescription
            type="Stack"
            id={stack_id}
            disabled={!canWrite}
          />
        </div>
        {/** NOTIFICATIONS */}
        <ResourceNotifications type="Stack" id={stack_id} />
      </div>

      <div className="mt-8 flex flex-col gap-12">
        {/* Actions */}
        {canExecute && (
          <Section title="Execute (Service)" icon={<Zap className="w-4 h-4" />}>
            <div className="flex gap-4 items-center flex-wrap">
              {Object.entries(Actions).map(([key, Action]) => (
                <Action key={key} id={stack_id} service={_service} />
              ))}
            </div>
          </Section>
        )}

        {/* Tabs */}
        <div className="pt-4">
          {stack && (
            <StackServiceTabs
              stack={stack}
              service={_service}
              container={container}
              swarm_service={swarm_service}
            />
          )}
        </div>
      </div>
    </div>
  );
};
