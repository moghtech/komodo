import { usePermissions, useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "@/ui/mobile-friendly-tabs";
import { useLocalStorage } from "@mantine/hooks";
import { useMemo } from "react";
import { useSwarm } from ".";
import SwarmConfig from "./config";
import { Tabs } from "@mantine/core";
import { colorByIntention, swarmStateIntention } from "@/lib/color";
import SwarmDockerResources from "./docker";
import InspectSection from "@/components/inspect-section";
import SwarmHostedResourcesSection from "./resources";

type SwarmTabsView = "Config" | "Docker" | "Resources" | "Inspect";

export default function SwarmTabs({ id }: { id: string }) {
  const [view, setView] = useLocalStorage<SwarmTabsView>({
    key: `swarm-${id}-tab-v1`,
    defaultValue: "Config",
  });

  const { specificInspect } = usePermissions({ type: "Swarm", id });

  const swarmInfo = useSwarm(id)?.info;

  const stacks =
    useRead("ListStacks", {}).data?.filter(
      (stack) => stack.info.swarm_id === id,
    ) ?? [];
  const noStacks = stacks.length === 0;
  const deployments =
    useRead("ListDeployments", {}).data?.filter(
      (deployment) => deployment.info.swarm_id === id,
    ) ?? [];
  const noDeployments = deployments.length === 0;
  const noResources = noDeployments && noStacks;

  const tabs = useMemo<TabNoContent[]>(
    () => [
      {
        value: "Config",
        icon: ICONS.Settings,
      },
      {
        value: "Docker",
        icon: ICONS.Docker,
      },
      {
        value: "Inspect",
        disabled: !specificInspect,
        icon: ICONS.Inspect,
      },
      {
        value: "Resources",
        icon: ICONS.Resources,
        disabled: noResources,
      },
    ],
    [specificInspect, noResources],
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
    case "Config":
      View = <SwarmConfig id={id} titleOther={Selector} />;
      break;
    case "Docker":
      View = <SwarmDockerResources id={id} titleOther={Selector} />;
      break;
    case "Inspect":
      View = (
        <InspectSection
          request={{
            type: "InspectSwarm",
            params: { swarm: id },
          }}
          titleOther={Selector}
        />
      );
      break;
    case "Resources":
      View = (
        <SwarmHostedResourcesSection
          swarmId={id}
          stacks={stacks}
          deployments={deployments}
          titleOther={Selector}
        />
      );
      break;
  }

  return (
    <Tabs
      color={colorByIntention(swarmStateIntention(swarmInfo?.state))}
      value={view}
    >
      {View}
    </Tabs>
  );
}
