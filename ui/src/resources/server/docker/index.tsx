import { atom, useAtom } from "jotai";
import { ReactNode, useMemo } from "react";
import { useServer } from "..";
import { Types } from "komodo_client";
import { useLocalStorage } from "@mantine/hooks";
import { Section } from "mogh_ui";
import { Center, Stack, Tabs, Text } from "@mantine/core";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "mogh_ui";
import { serverStateIntention } from "@/lib/color";
import ServerContainers from "./containers";
import ServerNetworks from "./networks";
import { ICONS } from "@/lib/icons";
import ServerVolumes from "./volumes";
import ServerImages from "./images";
import { useRead } from "@/lib/hooks";

type ServerDockerView = "Containers" | "Networks" | "Volumes" | "Images";

const searchAtom = atom("");
export function useServerDockerSearch() {
  return useAtom(searchAtom);
}

export default function ServerDockerResources({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const coreVersion = useRead("GetVersion", {}).data?.version;
  const info = useServer(id)?.info;
  const state = info?.state ?? Types.ServerState.NotOk;
  const [view, setView] = useLocalStorage<ServerDockerView>({
    key: "server-info-view-v1",
    defaultValue: "Containers",
  });

  if ([Types.ServerState.NotOk, Types.ServerState.Disabled].includes(state)) {
    return (
      <Section titleOther={titleOther}>
        <Center h="20vh">
          <Stack align="center" justify="center" gap="0">
            <Text fz="h2">Server unreachable</Text>
            <Text c="dimmed">Docker resources are not available</Text>
          </Stack>
        </Center>
      </Section>
    );
  }

  const tabsNoContent = useMemo<TabNoContent[]>(
    () => [
      {
        value: "Containers",
        icon: ICONS.Container,
      },
      {
        value: "Networks",
        icon: ICONS.Network,
      },
      {
        value: "Volumes",
        icon: ICONS.Volume,
      },
      {
        value: "Images",
        icon: ICONS.Image,
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
    case "Containers":
      View = <ServerContainers id={id} titleOther={Selector} />;
      break;
    case "Networks":
      View = <ServerNetworks id={id} titleOther={Selector} />;
      break;
    case "Volumes":
      View = <ServerVolumes id={id} titleOther={Selector} />;
      break;
    case "Images":
      View = <ServerImages id={id} titleOther={Selector} />;
      break;
  }

  return (
    <Section titleOther={titleOther}>
      <Tabs
        color={serverStateIntention(
          state,
          !!coreVersion && !!info?.version && coreVersion !== info.version,
        )}
        value={view}
      >
        {View}
      </Tabs>
    </Section>
  );
}
