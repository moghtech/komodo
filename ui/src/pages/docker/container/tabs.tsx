import { colorByIntention, containerStateIntention } from "@/lib/color";
import { usePermissions } from "@/lib/hooks";
import { useServer } from "@/resources/server";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "@/ui/mobile-friendly-tabs";
import { Tabs } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import { useMemo } from "react";
import { ICONS } from "@/theme/icons";
import TerminalSection from "@/components/terminal/section";
import LogSection from "@/components/log-section";
import InspectSection from "@/components/inspect-section";

export type ContainerTabsView = "Log" | "Inspect" | "Terminals";

export interface ContainerTabsProps {
  server: string;
  container: string;
  state: Types.ContainerStateStatusEnum;
  inspect: Types.Container | undefined;
}

export default function ContainerTabs({
  server,
  container,
  state,
  inspect,
}: ContainerTabsProps) {
  const [_view, setView] = useLocalStorage<ContainerTabsView>({
    key: `server-${server}-${container}-tabs-v2`,
    defaultValue: "Log",
  });
  const { specificLogs, specificInspect, specificTerminal } = usePermissions({
    type: "Server",
    id: server,
  });
  const containerTerminalsDisabled =
    useServer(server)?.info.container_terminals_disabled ?? true;
  const logDisabled =
    !specificLogs || state === Types.ContainerStateStatusEnum.Empty;
  const inspectDisabled =
    !specificInspect || state === Types.ContainerStateStatusEnum.Empty;
  const terminalDisabled =
    !specificTerminal ||
    containerTerminalsDisabled ||
    state !== Types.ContainerStateStatusEnum.Running;
  const view =
    (inspectDisabled && _view === "Inspect") ||
    (terminalDisabled && _view === "Terminals")
      ? "Log"
      : _view;

  const tabs = useMemo<TabNoContent[]>(
    () => [
      {
        value: "Log",
        hidden: !specificLogs,
        disabled: logDisabled,
        icon: ICONS.Log,
      },
      {
        value: "Inspect",
        hidden: !specificInspect,
        disabled: inspectDisabled,
        icon: ICONS.Inspect,
      },
      {
        value: "Terminals",
        hidden: !specificTerminal,
        disabled: terminalDisabled,
        icon: ICONS.Terminal,
      },
    ],
    [
      specificLogs,
      logDisabled,
      specificInspect,
      inspectDisabled,
      specificTerminal,
      terminalDisabled,
    ],
  );

  const Selector = (
    <MobileFriendlyTabsSelector
      tabs={tabs}
      value={view}
      onValueChange={setView as any}
      tabProps={{ w: 140 }}
    />
  );

  const target: Types.TerminalTarget = useMemo(
    () => ({
      type: "Container",
      params: {
        server,
        container,
      },
    }),
    [server, container],
  );

  let View = Selector;
  switch (view) {
    case "Log":
      View = (
        <LogSection
          target={{ type: "Container", serverId: server, container }}
          titleOther={Selector}
          disabled={logDisabled}
        />
      );
      break;
    case "Inspect":
      View = <InspectSection json={inspect} titleOther={Selector} />;
      break;
    case "Terminals":
      View = <TerminalSection target={target} titleOther={Selector} />;
      break;
  }

  return (
    <Tabs color={colorByIntention(containerStateIntention(state))} value={view}>
      {View}
    </Tabs>
  );
}
