import LogSection from "@/components/log-section";
import { colorByIntention, ColorIntention } from "@/lib/color";
import { usePermissions } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { MobileFriendlyTabsSelector } from "@/ui/mobile-friendly-tabs";
import { Tabs } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import { useMemo } from "react";
import InspectSection from "@/components/inspect-section";

type SwarmTaskTabsView = "Log" | "Inspect";

export interface SwarmTaskTabsProps {
  swarm: Types.SwarmListItem;
  task: string;
  intent: ColorIntention;
}

export default function SwarmTaskTabs({
  swarm,
  task,
  intent,
}: SwarmTaskTabsProps) {
  const [_view, setView] = useLocalStorage<SwarmTaskTabsView>({
    key: `swarm-${swarm.id}-task-${task}-tabs-v2`,
    defaultValue: "Log",
  });
  const { specificLogs, specificInspect } = usePermissions({
    type: "Swarm",
    id: swarm.id,
  });

  const view = !specificInspect && _view === "Inspect" ? "Log" : _view;

  const tabs = useMemo(
    () => [
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
    case "Log":
      View = (
        <LogSection
          target={{ type: "SwarmService", swarmId: swarm.id, service: task }}
          titleOther={Selector}
          disabled={!specificLogs}
        />
      );
      break;
    case "Inspect":
      View = (
        <InspectSection
          request={{
            type: "InspectSwarmTask",
            params: { swarm: swarm.id, task },
          }}
          titleOther={Selector}
        />
      );
      break;
  }

  return (
    <Tabs color={colorByIntention(intent)} value={view} mt="lg">
      {View}
    </Tabs>
  );
}
