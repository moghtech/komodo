import { colorByIntention, ColorIntention } from "@/lib/color";
import { usePermissions } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { MobileFriendlyTabsSelector } from "@/ui/mobile-friendly-tabs";
import { Tabs } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import { useMemo, useState } from "react";
import SwarmServiceTasksSection from "./tasks";
import LogSection from "@/components/log-section";
import SwarmServiceInspectSection from "./inspect";

type SwarmServiceTabsView = "Tasks" | "Log" | "Inspect";

export interface SwarmServiceTabsProps {
  swarm: Types.SwarmListItem;
  service: string;
  serviceId: string | undefined;
  intent: ColorIntention;
}

export default function SwarmServiceTabs({
  swarm,
  service,
  serviceId,
  intent,
}: SwarmServiceTabsProps) {
  const [_view, setView] = useLocalStorage<SwarmServiceTabsView>({
    key: `swarm-${swarm.id}-service-${service}-tabs-v2`,
    defaultValue: "Tasks",
  });
  const { specificLogs, specificInspect } = usePermissions({
    type: "Swarm",
    id: swarm.id,
  });
  const _search = useState("");

  const view =
    (!specificLogs && _view === "Log") ||
    (!specificInspect && _view === "Inspect")
      ? "Tasks"
      : _view;

  const tabs = useMemo(
    () => [
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
      tabProps={{ w: 140 }}
    />
  );

  let View = Selector;
  switch (view) {
    case "Tasks":
      View = (
        <SwarmServiceTasksSection
          id={swarm.id}
          serviceId={serviceId}
          titleOther={Selector}
          _search={_search}
        />
      );
      break;
    case "Log":
      View = (
        <LogSection
          target={{ type: "SwarmService", swarmId: swarm.id, service }}
          titleOther={Selector}
          disabled={!specificLogs}
        />
      );
      break;
    case "Inspect":
      View = (
        <SwarmServiceInspectSection
          swarm={swarm.id}
          service={service}
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
