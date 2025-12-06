import {
  ResourceDescription,
  ResourceLink,
  ResourcePageHeader,
} from "@components/resources/common";
import {
  useLocalStorage,
  usePermissions,
  useRead,
  useSetTitle,
} from "@lib/hooks";
import { Button } from "@ui/button";
import { ChevronLeft, Loader2 } from "lucide-react";
import { Link, useParams } from "react-router-dom";
import { MonacoEditor } from "@components/monaco";
import { SWARM_ICONS, SwarmResourceLink, useSwarm } from "@components/resources/swarm";
import { Types } from "komodo_client";
import {
  stroke_color_class_by_intention,
  swarm_state_intention,
} from "@lib/color";
import { ReactNode, useMemo } from "react";
import { MobileFriendlyTabsSelector } from "@ui/mobile-friendly-tabs";
import { SwarmServiceLogs } from "./log";
import { Section } from "@components/layouts";
import { ExportButton } from "@components/export";
import { ResourceNotifications } from "@pages/resource-notifications";

export default function SwarmTaskPage() {
  const { id, task: __task } = useParams() as {
    id: string;
    task: string;
  };
  const _task = decodeURIComponent(__task);
  const swarm = useSwarm(id);
  const { data: tasks, isPending } = useRead("ListSwarmTasks", {
    swarm: id,
  });
  const task = tasks?.find(
    (task) =>
      _task &&
      // Better to match on start to accept short ids too
      task.ID?.startsWith(_task)
  );
  const node = useRead("ListSwarmNodes", { swarm: id }).data?.find(
    (node) => node.ID === task?.NodeID
  );
  const service = useRead("ListSwarmServices", { swarm: id }).data?.find(
    (service) => service.ID === task?.ServiceID
  );
  const { canWrite } = usePermissions({
    type: "Swarm",
    id,
  });
  useSetTitle(`${swarm?.name} | Task | ${task?.ID ?? "Unknown"}`);

  if (isPending) {
    return (
      <div className="flex justify-center w-full py-4">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  if (!task) {
    return <div className="flex w-full py-4">Failed to inspect task.</div>;
  }

  const Icon = SWARM_ICONS.Task;
  const state =
    task.State === task.DesiredState
      ? Types.SwarmState.Healthy
      : Types.SwarmState.Unhealthy;
  const intention = swarm_state_intention(state);
  const strokeColor = stroke_color_class_by_intention(intention);

  return (
    <div>
      <div className="w-full flex items-center justify-between mb-12">
        <Link to={"/swarms/" + id}>
          <Button className="gap-2" variant="secondary">
            <ChevronLeft className="w-4" />
            Back
          </Button>
        </Link>
        <div className="flex items-center gap-4">
          <ExportButton targets={[{ type: "Swarm", id }]} />
        </div>
      </div>
      <div className="flex flex-col xl:flex-row gap-4">
        {/* HEADER */}
        <div className="w-full flex flex-col gap-4">
          <div className="flex flex-col gap-2 border rounded-md">
            <ResourcePageHeader
              type={undefined}
              id={undefined}
              intent={intention}
              icon={<Icon size={8} className={strokeColor} />}
              resource={undefined}
              name={task.ID}
              state={state}
              status={task.State}
            />
            <div className="flex flex-col pb-2 px-4">
              <div className="flex items-center gap-x-4 gap-y-0 flex-wrap text-muted-foreground">
                <div>Swarm Task</div>
                |
                <ResourceLink type="Swarm" id={id} />
                |
                <SwarmResourceLink
                  type="Service"
                  swarm_id={id}
                  resource_id={service?.ID}
                  name={service?.Name}
                />
                |
                <SwarmResourceLink
                  type="Node"
                  swarm_id={id}
                  resource_id={node?.ID}
                  name={node?.Hostname}
                />
              </div>
            </div>
          </div>
          <ResourceDescription type="Swarm" id={id} disabled={!canWrite} />
        </div>
        {/** NOTIFICATIONS */}
        <ResourceNotifications type="Swarm" id={id} />
      </div>

      <div className="mt-8 flex flex-col gap-12">
        {/* Actions */}

        {/* Tabs */}
        <div className="pt-4">
          {swarm && <SwarmTaskTabs swarm={swarm} task={_task} />}
        </div>
      </div>
    </div>
  );
}

type SwarmTaskTabsView = "Log" | "Inspect";

const SwarmTaskTabs = ({
  swarm,
  task,
}: {
  swarm: Types.SwarmListItem;
  task: string;
}) => {
  const [_view, setView] = useLocalStorage<SwarmTaskTabsView>(
    `swarm-${swarm.id}-task-${task}-tabs-v2`,
    "Log"
  );
  const { specificLogs, specificInspect } = usePermissions({
    type: "Swarm",
    id: swarm.id,
  });

  const view = !specificInspect && _view === "Inspect" ? "Log" : _view;

  const tabs = useMemo(
    () => [
      {
        value: "Log",
        disabled: !specificLogs,
      },
      {
        value: "Inspect",
        disabled: !specificInspect,
      },
    ],
    [specificLogs, specificInspect]
  );

  const Selector = (
    <MobileFriendlyTabsSelector
      tabs={tabs}
      value={view}
      onValueChange={setView as any}
      tabsTriggerClassname="w-[110px]"
    />
  );

  switch (view) {
    case "Log":
      return (
        <SwarmServiceLogs
          id={swarm.id}
          service={task}
          titleOther={Selector}
          disabled={!specificLogs}
        />
      );
    case "Inspect":
      return (
        <SwarmTaskInspect swarm={swarm.id} task={task} titleOther={Selector} />
      );
  }
};

const SwarmTaskInspect = ({
  swarm,
  task,
  titleOther,
}: {
  swarm: string;
  task: string;
  titleOther: ReactNode;
}) => {
  const { data: inspect, isPending } = useRead("InspectSwarmTask", {
    swarm,
    task,
  });

  if (isPending) {
    return (
      <div className="flex justify-center w-full py-4">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  if (!task) {
    return <div className="flex w-full py-4">Failed to inspect task.</div>;
  }

  return (
    <Section titleOther={titleOther}>
      <MonacoEditor
        value={JSON.stringify(inspect, null, 2)}
        language="json"
        readOnly
      />
    </Section>
  );
};
