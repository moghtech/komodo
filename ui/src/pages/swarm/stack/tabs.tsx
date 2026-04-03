import { usePermissions } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { MobileFriendlyTabsSelector } from "@/ui/mobile-friendly-tabs";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import { useMemo, useState } from "react";
import SwarmServicesSection from "@/components/swarm/services-section";
import SwarmTasksSection from "@/components/swarm/tasks-section";
import { colorByIntention, ColorIntention } from "@/lib/color";
import { Tabs } from "@mantine/core";
import SwarmStackLogsSection from "./logs";
import InspectSection from "@/components/inspect-section";

type SwarmStackTabsView = "Services" | "Tasks" | "Log" | "Inspect";

export interface SwarmStackTabsProps {
  swarm: Types.SwarmListItem;
  stack: Types.SwarmStack;
  intent: ColorIntention;
}

export default function SwarmStackTabs({
  swarm,
  stack,
  intent,
}: SwarmStackTabsProps) {
  const [_view, setView] = useLocalStorage<SwarmStackTabsView>({
    key: `swarm-${swarm.id}-stack-${stack}-tabs-v2`,
    defaultValue: "Services",
  });
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
        icon: ICONS.SwarmService,
      },
      {
        value: "Tasks",
        icon: ICONS.SwarmTask,
      },
      {
        value: "Log",
        icon: ICONS.Log,
        disabled: !specificLogs,
      },
      {
        value: "Inspect",
        icon: ICONS.Inspect,
        disabled: !specificInspect,
      },
    ],
    [specificLogs, specificInspect],
  );

  const Selector = (
    <MobileFriendlyTabsSelector
      tabs={tabs}
      value={view}
      onValueChange={setView as any}
    />
  );

  let View = Selector;
  switch (view) {
    case "Services":
      View = (
        <SwarmServicesSection
          id={swarm.id}
          services={stack.Services}
          titleOther={Selector}
          _search={_search}
        />
      );
      break;
    case "Tasks":
      View = (
        <SwarmTasksSection
          id={swarm.id}
          tasks={stack.Tasks}
          titleOther={Selector}
          _search={_search}
        />
      );
      break;
    case "Log":
      View = (
        <SwarmStackLogsSection
          swarmId={swarm.id}
          stack={stack}
          disabled={!specificLogs}
          titleOther={Selector}
        />
      );
      break;
    case "Inspect":
      View = (
        <InspectSection
          request={{
            type: "InspectSwarmStack",
            params: { swarm: swarm.id, stack: stack.Name },
          }}
          titleOther={Selector}
        />
      );
      break;
  }

  return (
    <Tabs color={colorByIntention(intent)} value={view}>
      {View}
    </Tabs>
  );
}
