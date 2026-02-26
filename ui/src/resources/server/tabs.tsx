import { usePermissions, useRead } from "@/lib/hooks";
import { useLocalStorage } from "@mantine/hooks";
import { useServer } from ".";
import { useMemo } from "react";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "@/ui/mobile-friendly-tabs";
import { Types } from "komodo_client";
import TerminalSection from "@/components/terminal/section";
import { Tabs } from "@mantine/core";
import { colorByIntention, serverStateIntention } from "@/lib/color";
import ServerConfig from "./config";
import { ICONS } from "@/theme/icons";
import ServerDockerResources from "./docker";
import ServerStats from "./stats";
import ServerHostedResourcesSection from "./resources";

type ServerTabsView = "Config" | "Stats" | "Docker" | "Resources" | "Terminals";

export default function ServerTabs({ id }: { id: string }) {
  const [view, setView] = useLocalStorage<ServerTabsView>({
    key: `server-${id}-tab-v2`,
    defaultValue: "Config",
  });

  const { specificTerminal } = usePermissions({ type: "Server", id });

  const serverInfo = useServer(id)?.info;
  const notReachable = serverInfo?.state !== Types.ServerState.Ok;
  const terminalDisabled =
    !specificTerminal || (serverInfo?.terminals_disabled ?? true);

  const stacks =
    useRead("ListStacks", {}).data?.filter(
      (stack) => stack.info.server_id === id,
    ) ?? [];
  const noStacks = stacks.length === 0;
  const deployments =
    useRead("ListDeployments", {}).data?.filter(
      (deployment) => deployment.info.server_id === id,
    ) ?? [];
  const noDeployments = deployments.length === 0;
  const repos =
    useRead("ListRepos", {}).data?.filter(
      (repo) => repo.info.server_id === id,
    ) ?? [];
  const noRepos = repos.length === 0;

  const noResources = noDeployments && noRepos && noStacks;

  const tabs = useMemo<TabNoContent[]>(
    () => [
      {
        value: "Config",
        icon: ICONS.Settings,
      },
      {
        value: "Stats",
        icon: ICONS.Stats,
      },
      {
        value: "Docker",
        icon: ICONS.Docker,
        disabled: notReachable,
      },
      {
        value: "Resources",
        icon: ICONS.Resources,
        disabled: noResources,
      },
      {
        value: "Terminals",
        disabled: terminalDisabled,
        icon: ICONS.Terminal,
      },
    ],
    [noResources, terminalDisabled],
  );

  const Selector = (
    <MobileFriendlyTabsSelector
      tabs={tabs}
      value={view}
      onValueChange={setView as any}
    />
  );

  const target: Types.TerminalTarget = useMemo(
    () => ({
      type: "Server",
      params: {
        server: id,
      },
    }),
    [id],
  );

  let View = Selector;
  switch (view) {
    case "Config":
      View = <ServerConfig id={id} titleOther={Selector} />;
      break;
    case "Stats":
      View = <ServerStats id={id} titleOther={Selector} />;
      break;
    case "Docker":
      View = <ServerDockerResources id={id} titleOther={Selector} />;
      break;
    case "Resources":
      View = (
        <ServerHostedResourcesSection
          serverId={id}
          stacks={stacks}
          deployments={deployments}
          repos={repos}
          titleOther={Selector}
        />
      );
      break;
    case "Terminals":
      View = <TerminalSection target={target} titleOther={Selector} />;
      break;
  }

  return (
    <Tabs
      color={colorByIntention(serverStateIntention(serverInfo?.state))}
      value={view}
    >
      {View}
    </Tabs>
  );
}
