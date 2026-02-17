import { colorByIntention, ColorIntention } from "@/lib/color";
import { usePermissions } from "@/lib/hooks";
import { MobileFriendlyTabsSelector } from "@/ui/mobile-friendly-tabs";
import { Tabs } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import { useMemo, useState } from "react";
import SwarmNodeTasksSection from "./tasks";
import SwarmNodeInspectSection from "./inspect";
import { ICONS } from "@/theme/icons";

type SwarmNodeTabsView = "Tasks" | "Inspect";

export interface SwarmNodeTabsProps {
  swarm: Types.SwarmListItem;
  _node: string;
  node: Types.SwarmNode;
  intent: ColorIntention;
}

export default function SwarmNodeTabs({
  swarm,
  _node,
  node,
  intent,
}: SwarmNodeTabsProps) {
  const [_view, setView] = useLocalStorage<SwarmNodeTabsView>({
    key: `swarm-${swarm.id}-node-${node}-tabs-v1`,
    defaultValue: "Tasks",
  });
  const { specificInspect } = usePermissions({
    type: "Swarm",
    id: swarm.id,
  });
  const _search = useState("");

  const view = !specificInspect && _view === "Inspect" ? "Tasks" : _view;

  const tabs = useMemo(
    () => [
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
      tabProps={{ w: 140 }}
    />
  );

  let View = Selector;

  switch (view) {
    case "Inspect":
      View = (
        <SwarmNodeInspectSection
          swarm={swarm.id}
          node={_node}
          titleOther={Selector}
        />
      );
      break;
    case "Tasks":
      View = (
        <SwarmNodeTasksSection
          id={swarm.id}
          nodeId={node.ID}
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
