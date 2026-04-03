import { atom, useAtom } from "jotai";
import { ReactNode, useMemo } from "react";
import { useSwarm } from "..";
import { Types } from "komodo_client";
import { useLocalStorage } from "@mantine/hooks";
import { Section } from "mogh_ui";
import { Center, Stack, Tabs, Text } from "@mantine/core";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "mogh_ui";
import { swarmStateIntention } from "@/lib/color";
import { ICONS } from "@/lib/icons";
import SwarmNodes from "./nodes";
import SwarmStacks from "./stacks";
import SwarmConfigs from "./configs";
import SwarmSecrets from "./secrets";
import SwarmServices from "./services";
import SwarmTasks from "./tasks";

type SwarmDockerView =
  | "Nodes"
  | "Stacks"
  | "Services"
  | "Tasks"
  | "Configs"
  | "Secrets";

const searchAtom = atom("");
export function useSwarmDockerSearch() {
  return useAtom(searchAtom);
}

export default function SwarmDockerResources({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const state = useSwarm(id)?.info.state ?? Types.SwarmState.Unknown;
  const [view, setView] = useLocalStorage<SwarmDockerView>({
    key: "swarm-info-view-v1",
    defaultValue: "Nodes",
  });

  if (state === Types.SwarmState.Unknown) {
    return (
      <Section titleOther={titleOther}>
        <Center h="20vh">
          <Stack align="center" justify="center" gap="0">
            <Text fz="h2">Swarm unreachable</Text>
            <Text c="dimmed">Docker resources are not available</Text>
          </Stack>
        </Center>
      </Section>
    );
  }

  const tabsNoContent = useMemo<TabNoContent[]>(
    () => [
      {
        value: "Nodes",
        icon: ICONS.SwarmNode,
      },
      {
        value: "Stacks",
        icon: ICONS.SwarmStack,
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
        value: "Configs",
        icon: ICONS.SwarmConfig,
      },
      {
        value: "Secrets",
        icon: ICONS.SwarmSecret,
      },
    ],
    [],
  );

  const Selector = (
    <MobileFriendlyTabsSelector
      tabs={tabsNoContent}
      value={view}
      onValueChange={setView as any}
    />
  );

  let View = Selector;
  switch (view) {
    case "Nodes":
      View = <SwarmNodes id={id} titleOther={Selector} />;
      break;
    case "Stacks":
      View = <SwarmStacks id={id} titleOther={Selector} />;
      break;
    case "Services":
      View = <SwarmServices id={id} titleOther={Selector} />;
      break;
    case "Tasks":
      View = <SwarmTasks id={id} titleOther={Selector} />;
      break;
    case "Configs":
      View = <SwarmConfigs id={id} titleOther={Selector} />;
      break;
    case "Secrets":
      View = <SwarmSecrets id={id} titleOther={Selector} />;
      break;
  }

  return (
    <Section titleOther={titleOther}>
      <Tabs color={swarmStateIntention(state)} value={view}>
        {View}
      </Tabs>
    </Section>
  );
}
