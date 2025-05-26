import { Section } from "@components/layouts";
import {
  ResourceDescription,
  ResourceLink,
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
} from "@lib/color";
import {
  usePermissions,
  useLocalStorage,
  useRead,
  useSetTitle,
} from "@lib/hooks";
import { cn } from "@lib/utils";
import { Types } from "komodo_client";
import { ChevronLeft, Clapperboard, Layers2 } from "lucide-react";
import { Link, useParams } from "react-router-dom";
import { StackServiceLogs } from "./log";
import { Button } from "@ui/button";
import { ExportButton } from "@components/export";
import { DockerResourceLink, ResourcePageHeader } from "@components/util";
import { ResourceNotifications } from "@pages/resource-notifications";
import { Fragment } from "react/jsx-runtime";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@ui/tabs";
import { ContainerTerminal } from "@components/terminal";
import { useServer } from "@components/resources/server";

type IdServiceComponent = React.FC<{ id: string; service?: string }>;

const Actions: { [action: string]: IdServiceComponent } = {
  DeployStack,
  PullStack,
  RestartStack,
  PauseUnpauseStack,
  StartStopStack,
  DestroyStack,
};

export const StackServicePage = () => {
  const { type, id, service } = useParams() as {
    type: string;
    id: string;
    service: string;
  };
  if (type !== "stacks") {
    return <div>This resource type does not have any services.</div>;
  }
  return <StackServicePageInner stack_id={id} service={service} />;
};

const StackServicePageInner = ({
  stack_id,
  service,
}: {
  stack_id: string;
  service: string;
}) => {
  const stack = useStack(stack_id);
  useSetTitle(`${stack?.name} | ${service}`);
  const { canExecute, canWrite } = usePermissions({
    type: "Stack",
    id: stack_id,
  });
  const services = useRead("ListStackServices", { stack: stack_id }).data;
  const container = services?.find((s) => s.service === service)?.container;
  const state = container?.state ?? Types.ContainerStateStatusEnum.Empty;
  const intention = container_state_intention(state);
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
        {/** HEADER */}
        <div className="w-full flex flex-col gap-4">
          <div className="flex flex-col gap-2 border rounded-md">
            {/* <Components.ResourcePageHeader id={id} /> */}
            <ResourcePageHeader
              type={undefined}
              id={undefined}
              intent={intention}
              icon={<Layers2 className={cn("w-8 h-8", stroke_color)} />}
              name={service}
              state={state}
              status={container?.status}
            />
            <div className="flex flex-col pb-2 px-4">
              <div className="flex items-center gap-x-4 gap-y-0 flex-wrap text-muted-foreground">
                <ResourceLink type="Stack" id={stack_id} />
                {stack?.info.server_id && (
                  <>
                    |
                    <ResourceLink type="Server" id={stack.info.server_id} />
                  </>
                )}
                {stack?.info.server_id && container?.name && (
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
                {stack?.info.server_id && container?.image && (
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
                {stack?.info.server_id &&
                  container?.networks.map((network) => (
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
                {stack?.info.server_id &&
                  container &&
                  container.volumes.map((volume) => (
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
          <Section
            title="Actions (Service)"
            icon={<Clapperboard className="w-4 h-4" />}
          >
            <div className="flex gap-4 items-center flex-wrap">
              {Object.entries(Actions).map(([key, Action]) => (
                <Action key={key} id={stack_id} service={service} />
              ))}
            </div>
          </Section>
        )}

        {/* Logs */}
        <div className="pt-4">
          {stack && (
            <LogOrTerminal
              stack={stack}
              service={service}
              container_name={container?.name}
              container_state={state}
            />
          )}
        </div>
      </div>
    </div>
  );
};

const LogOrTerminal = ({
  stack,
  service,
  container_name,
  container_state,
}: {
  stack: Types.StackListItem;
  service: string;
  container_name: string | undefined;
  container_state: Types.ContainerStateStatusEnum;
}) => {
  const [_view, setView] = useLocalStorage<"Log" | "Terminal">(
    `stack-${stack.id}-${service}-tabs-v1`,
    "Log"
  );
  const { specific } = usePermissions({
    type: "Stack",
    id: stack.id,
  });
  const container_exec_disabled =
    useServer(stack.info.server_id)?.info.container_exec_disabled ?? true;
  const terminalDisabled =
    !specific.includes(Types.SpecificPermission.Terminal) ||
    container_exec_disabled ||
    container_state !== Types.ContainerStateStatusEnum.Running;
  const logDisabled =
    !specific.includes(Types.SpecificPermission.DockerLog) ||
    container_state === Types.ContainerStateStatusEnum.Empty;
  const view = terminalDisabled && _view === "Terminal" ? "Log" : _view;
  const tabs = (
    <TabsList className="justify-start w-fit">
      <TabsTrigger value="Log" className="w-[110px]" disabled={logDisabled}>
        Log
      </TabsTrigger>
      {!terminalDisabled && (
        <TabsTrigger value="Terminal" className="w-[110px]">
          Terminal
        </TabsTrigger>
      )}
    </TabsList>
  );
  return (
    <Tabs value={view} onValueChange={setView as any} className="grid gap-4">
      <TabsContent value="Log">
        <StackServiceLogs
          id={stack.id}
          service={service}
          titleOther={tabs}
          disabled={logDisabled}
        />
      </TabsContent>
      <TabsContent value="Terminal">
        {stack.info.server_id && container_name && (
          <ContainerTerminal
            server={stack.info.server_id}
            container={container_name}
            titleOther={tabs}
          />
        )}
      </TabsContent>
    </Tabs>
  );
};
