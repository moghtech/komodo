import { colorByIntention, ColorIntention } from "@/lib/color";
import { usePermissions } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "mogh_ui";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import { useMemo, useState } from "react";
import SwarmConfigTasksSection from "./tasks";
import SwarmConfigServicesSection from "./services";
import { Tabs } from "@mantine/core";
import SwarmConfigEditSection from "./edit";
import InspectSection from "@/components/inspect-section";

type SwarmConfigTabsView = "Edit" | "Services" | "Tasks" | "Inspect";

export interface SwarmConfigTabsProps {
  swarm: Types.SwarmListItem;
  config: string;
  intent: ColorIntention;
}

export default function SwarmConfigTabs({
  swarm,
  config,
  intent,
}: SwarmConfigTabsProps) {
  const [_view, setView] = useLocalStorage<SwarmConfigTabsView>({
    key: `swarm-${swarm.id}-config-${config}-tabs-v2`,
    defaultValue: "Edit",
  });
  const { specificInspect } = usePermissions({
    type: "Swarm",
    id: swarm.id,
  });
  const _search = useState("");

  const view = !specificInspect && _view === "Inspect" ? "Edit" : _view;

  const tabs = useMemo<TabNoContent[]>(
    () => [
      {
        value: "Edit",
        icon: ICONS.Edit,
      },
      {
        value: "Services",
        icon: ICONS.SwarmService,
      },
      {
        value: "Tasks",
        icon: ICONS.SwarmTask,
      },
      {
        value: "Inspect",
        icon: ICONS.Inspect,
        disabled: !specificInspect,
      },
    ],
    [specificInspect],
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
    case "Edit":
      View = (
        <SwarmConfigEditSection
          swarm={swarm.id}
          config={config}
          titleOther={Selector}
        />
      );
      break;
    case "Inspect":
      View = (
        <InspectSection
          request={{
            type: "InspectSwarmConfig",
            params: { swarm: swarm.id, config },
          }}
          titleOther={Selector}
        />
      );
      break;
    case "Services":
      View = (
        <SwarmConfigServicesSection
          id={swarm.id}
          config={config}
          titleOther={Selector}
          _search={_search}
        />
      );
      break;
    case "Tasks":
      View = (
        <SwarmConfigTasksSection
          id={swarm.id}
          config={config}
          titleOther={Selector}
          _search={_search}
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
