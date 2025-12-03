import {
  ResourceDescription,
  ResourceLink,
  ResourcePageHeader,
} from "@components/resources/common";
import {
  useExecute,
  useLocalStorage,
  usePermissions,
  useRead,
  useSetTitle,
} from "@lib/hooks";
import { Button } from "@ui/button";
import { ChevronLeft, Clapperboard, Loader2, Trash } from "lucide-react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { SWARM_ICONS, useSwarm } from "@components/resources/swarm";
import {
  stroke_color_class_by_intention,
  swarm_state_intention,
} from "@lib/color";
import { ExportButton } from "@components/export";
import { ResourceNotifications } from "@pages/resource-notifications";
import { Types } from "komodo_client";
import { ReactNode, useMemo, useState } from "react";
import { MobileFriendlyTabsSelector } from "@ui/mobile-friendly-tabs";
import { Section } from "@components/layouts";
import { MonacoEditor } from "@components/monaco";
import {
  SwarmStackServicesTable,
  SwarmStackTasksTable,
} from "@components/resources/swarm/table";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { SwarmServiceLogs } from "./log";
import { ActionWithDialog } from "@components/util";

export default function SwarmStackPage() {
  const { id, stack: __stack } = useParams() as {
    id: string;
    stack: string;
  };
  const _stack = decodeURIComponent(__stack);
  const swarm = useSwarm(id);
  const { data: stack, isPending } = useRead("InspectSwarmStack", {
    swarm: id,
    stack: _stack,
  });
  const { canWrite, canExecute } = usePermissions({
    type: "Swarm",
    id,
  });
  useSetTitle(`${swarm?.name} | Stack | ${stack?.Name ?? "Unknown"}`);

  if (isPending) {
    return (
      <div className="flex justify-center w-full py-4">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  if (!stack) {
    return <div className="flex w-full py-4">Failed to inspect stack.</div>;
  }

  const Icon = SWARM_ICONS.Stack;
  const state = stack.State;
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
              name={stack.Name}
              state={state}
              status={`${stack.Services.length} Services`}
            />
            <div className="flex flex-col pb-2 px-4">
              <div className="flex items-center gap-x-4 gap-y-0 flex-wrap text-muted-foreground">
                <ResourceLink type="Swarm" id={id} />
                <div>|</div>
                <div>Swarm Stack</div>
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
        {canExecute && (
          <Section title="Actions" icon={<Clapperboard className="w-4 h-4" />}>
            <div className="flex gap-4 items-center flex-wrap">
              <RemoveStack id={id} stack={stack.Name} />
            </div>
          </Section>
        )}

        {/* Tabs */}
        <div className="pt-4">
          {swarm && <SwarmStackTabs swarm={swarm} stack={stack} />}
        </div>
      </div>
    </div>
  );
}

/* ACTIONS */

const RemoveStack = ({ id, stack }: { id: string; stack: string }) => {
  const nav = useNavigate();
  const { mutate: remove, isPending } = useExecute("RemoveSwarmStacks", {
    onSuccess: () => nav("/swarms/" + id),
  });

  return (
    <ActionWithDialog
      name={stack}
      title="Remove"
      icon={<Trash className="h-4 w-4" />}
      onClick={() => remove({ swarm: id, stacks: [stack], detach: false })}
      disabled={isPending}
      loading={isPending}
    />
  );
};

/* TABS */
type SwarmStackTabsView = "Services" | "Tasks" | "Log" | "Inspect";

const SwarmStackTabs = ({
  swarm,
  stack,
}: {
  swarm: Types.SwarmListItem;
  stack: Types.SwarmStack;
}) => {
  const [_view, setView] = useLocalStorage<SwarmStackTabsView>(
    `swarm-${swarm.id}-stack-${stack}-tabs-v2`,
    "Services"
  );
  const _search = useState("");
  const { specificInspect, specificLogs } = usePermissions({
    type: "Swarm",
    id: swarm.id,
  });

  const view =
    (!specificLogs && _view === "Log") ||
    (!specificInspect && _view === "Inspect")
      ? "Services"
      : _view;

  const tabs = useMemo(
    () => [
      {
        value: "Services",
      },
      {
        value: "Tasks",
      },
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
    case "Services":
      return (
        <SwarmStackServicesTable
          id={swarm.id}
          services={stack.Services}
          titleOther={Selector}
          _search={_search}
        />
      );
    case "Tasks":
      return (
        <SwarmStackTasksTable
          id={swarm.id}
          tasks={stack.Tasks}
          titleOther={Selector}
          _search={_search}
        />
      );
    case "Log":
      return (
        <SwarmStackLogs
          id={swarm.id}
          stack={stack}
          disabled={!specificLogs}
          titleOther={Selector}
        />
      );
    case "Inspect":
      return (
        <SwarmStackInspect
          stack={stack}
          titleOther={Selector}
          disabled={!specificInspect}
        />
      );
  }
};

const SwarmStackLogs = ({
  id,
  stack,
  disabled,
  titleOther,
}: {
  id: string;
  stack: Types.SwarmStack;
  disabled: boolean;
  titleOther: ReactNode;
}) => {
  const [service, setService] = useState(stack.Services[0].Name ?? "");
  return (
    <SwarmServiceLogs
      id={id}
      service={service}
      titleOther={titleOther}
      disabled={disabled}
      extraParams={
        <Select value={service} onValueChange={setService}>
          <SelectTrigger className="w-fit">
            <div className="flex items-center gap-2 pr-2">
              <div className="text-xs text-muted-foreground">Service:</div>
              <SelectValue placeholder="Select Service" />
            </div>
          </SelectTrigger>
          <SelectContent>
            {stack.Services.filter((service) => service.Name).map((service) => (
              <SelectItem key={service.Name} value={service.Name!}>
                {service.Name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      }
    />
  );
};

const SwarmStackInspect = ({
  stack,
  titleOther,
  disabled,
}: {
  stack: Types.SwarmStack;
  titleOther: ReactNode;
  disabled: boolean;
}) => {
  return (
    <Section titleOther={titleOther}>
      {disabled ? (
        <div>User does not have Inspect permission on Swarm.</div>
      ) : (
        <MonacoEditor
          value={JSON.stringify(stack, null, 2)}
          language="json"
          readOnly
        />
      )}
    </Section>
  );
};
