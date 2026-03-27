import { colorByIntention, ColorIntention } from "@/lib/color";
import { usePermissions } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "@/ui/mobile-friendly-tabs";
import { Tabs } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import { useMemo, useState } from "react";
import SwarmSecretServicesSection from "./services";
import SwarmSecretTasksSection from "./tasks";
import SwarmSecretEditSection from "./edit";
import InspectSection from "@/components/inspect-section";

type SwarmSecretTabsView = "Edit" | "Services" | "Tasks" | "Inspect";

export interface SwarmSecretTabsProps {
  swarm: Types.SwarmListItem;
  secret: string;
  intent: ColorIntention;
}

export default function SwarmSecretTabs({
  swarm,
  secret,
  intent,
}: SwarmSecretTabsProps) {
  const [_view, setView] = useLocalStorage<SwarmSecretTabsView>({
    key: `swarm-${swarm.id}-secret-${secret}-tabs-v2`,
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
        <SwarmSecretEditSection
          swarm={swarm.id}
          secret={secret}
          titleOther={Selector}
        />
      );
      break;
    case "Inspect":
      View = (
        <InspectSection
          request={{
            type: "InspectSwarmSecret",
            params: { swarm: swarm.id, secret },
          }}
          titleOther={Selector}
        />
      );
      break;
    case "Services":
      View = (
        <SwarmSecretServicesSection
          id={swarm.id}
          secret={secret}
          titleOther={Selector}
          _search={_search}
        />
      );
      break;
    case "Tasks":
      View = (
        <SwarmSecretTasksSection
          id={swarm.id}
          secret={secret}
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
