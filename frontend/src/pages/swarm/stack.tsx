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
  const { canWrite } = usePermissions({
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

        {/* Tabs */}
        <div className="pt-4">
          {swarm && <SwarmStackTabs swarm={swarm} stack={stack} />}
        </div>
      </div>
    </div>
  );
}

type SwarmStackTabsView = "Services" | "Tasks" | "Inspect";

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
  const { specificInspect } = usePermissions({
    type: "Swarm",
    id: swarm.id,
  });

  const view = !specificInspect && _view === "Inspect" ? "Log" : _view;

  const tabs = useMemo(
    () => [
      {
        value: "Services",
      },
      {
        value: "Tasks",
      },
      {
        value: "Inspect",
        disabled: !specificInspect,
      },
    ],
    [specificInspect]
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
    case "Inspect":
      return <SwarmStackInspect stack={stack} titleOther={Selector} />;
  }
};

const SwarmStackInspect = ({
  stack,
  titleOther,
}: {
  stack: Types.SwarmStack;
  titleOther: ReactNode;
}) => {
  return (
    <Section titleOther={titleOther}>
      <MonacoEditor
        value={JSON.stringify(stack, null, 2)}
        language="json"
        readOnly
      />
    </Section>
  );
};
