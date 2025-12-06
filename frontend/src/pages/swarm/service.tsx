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
import { ChevronLeft, Loader2, Zap } from "lucide-react";
import { Link, useParams } from "react-router-dom";
import { MonacoEditor } from "@components/monaco";
import { SWARM_ICONS, useSwarm } from "@components/resources/swarm";
import { ExportButton } from "@components/export";
import { Types } from "komodo_client";
import {
  stroke_color_class_by_intention,
  swarm_state_intention,
} from "@lib/color";
import { ResourceNotifications } from "@pages/resource-notifications";
import { ReactNode, useMemo } from "react";
import { MobileFriendlyTabsSelector } from "@ui/mobile-friendly-tabs";
import { SwarmServiceLogs } from "./log";
import { Section } from "@components/layouts";
import { RemoveSwarmResource } from "./remove";

export default function SwarmServicePage() {
  const { id, service: __service } = useParams() as {
    id: string;
    service: string;
  };
  const _service = decodeURIComponent(__service);
  const swarm = useSwarm(id);
  const { data: services, isPending } = useRead("ListSwarmServices", {
    swarm: id,
  });
  const service = services?.find(
    (service) =>
      _service &&
      // First match on name here.
      // Then better to match on ID start to accept short ids too.
      (service.Name === _service || service.ID?.startsWith(_service))
  );
  const tasks =
    useRead("ListSwarmTasks", {
      swarm: id,
    }).data?.filter((task) => service?.ID && task.ServiceID === service.ID) ??
    [];
  const { canWrite, canExecute } = usePermissions({
    type: "Swarm",
    id,
  });
  useSetTitle(
    `${swarm?.name} | Service | ${service?.Name ?? service?.ID ?? "Unknown"}`
  );

  if (isPending) {
    return (
      <div className="flex justify-center w-full py-4">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  if (!service) {
    return <div className="flex w-full py-4">Failed to inspect service.</div>;
  }

  const Icon = SWARM_ICONS.Service;
  const state = get_service_state_from_tasks(tasks);
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
              name={service.Name}
              state={state}
              status={`${tasks.length} Task${tasks.length === 1 ? "" : "s"}`}
            />
            <div className="flex flex-col pb-2 px-4">
              <div className="flex items-center gap-x-4 gap-y-0 flex-wrap text-muted-foreground">
                <ResourceLink type="Swarm" id={id} />
                <div>|</div>
                <div>Swarm Service</div>
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
        {canExecute && service.Name && (
          <Section title="Execute" icon={<Zap className="w-4 h-4" />}>
            <div className="flex gap-4 items-center flex-wrap">
              <RemoveSwarmResource
                id={id}
                type="Service"
                resource_id={service.Name}
              />
            </div>
          </Section>
        )}

        {/* Tabs */}
        <div className="pt-4">
          {swarm && <SwarmServiceTabs swarm={swarm} service={_service} />}
        </div>
      </div>
    </div>
  );
}

type SwarmServiceTabsView = "Log" | "Inspect";

const SwarmServiceTabs = ({
  swarm,
  service,
}: {
  swarm: Types.SwarmListItem;
  service: string;
}) => {
  const [_view, setView] = useLocalStorage<SwarmServiceTabsView>(
    `swarm-${swarm.id}-service-${service}-tabs-v2`,
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
          service={service}
          titleOther={Selector}
          disabled={!specificLogs}
        />
      );
    case "Inspect":
      return (
        <SwarmServiceInspect
          swarm={swarm.id}
          service={service}
          titleOther={Selector}
        />
      );
  }
};

const SwarmServiceInspect = ({
  swarm,
  service,
  titleOther,
}: {
  swarm: string;
  service: string;
  titleOther: ReactNode;
}) => {
  const { data: inspect, isPending } = useRead("InspectSwarmService", {
    swarm,
    service,
  });

  if (isPending) {
    return (
      <div className="flex justify-center w-full py-4">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  if (!service) {
    return <div className="flex w-full py-4">Failed to inspect service.</div>;
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

const get_service_state_from_tasks = (
  tasks: Types.SwarmTaskListItem[]
): Types.SwarmState => {
  for (const task of tasks) {
    if (task.State !== task.DesiredState) {
      return Types.SwarmState.Unhealthy;
    }
  }
  return Types.SwarmState.Healthy;
};
