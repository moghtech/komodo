import { useLocalStorage } from "@mantine/hooks";
import { useStack } from ".";
import { usePermissions, useRead } from "@/lib/hooks";
import { Types } from "komodo_client";
import { useMemo } from "react";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "@/ui/mobile-friendly-tabs";
import { Tabs } from "@mantine/core";
import { ICONS } from "@/theme/icons";
import { colorByIntention, stackStateIntention } from "@/lib/color";
import { useServer } from "@/resources/server";
import StackConfig from "./config";
import StackInfo from "./info";
import StackServices from "./services";
import StackLog from "./log";
import TerminalSection from "@/components/terminal/section";
import StackContainerStats from "./stats/container";

type StackTabsView = "Config" | "Info" | "Services" | "Log" | "Terminals" | "Stats";

export default function StackTabs({ id }: { id: string }) {
  const [_view, setView] = useLocalStorage<StackTabsView>({
    key: `stack-${id}-tab-v2`,
    defaultValue: "Config",
  });
  const info = useStack(id)?.info;
  const { specificLogs, specificTerminal } = usePermissions({
    type: "Stack",
    id,
  });

  const services = useRead("ListStackServices", { stack: id }).data;

  const containerTerminalsDisabled =
    useServer(info?.server_id)?.info.container_terminals_disabled ?? false;

  const state = info?.state;
  const hideInfo = !info?.files_on_host && !info?.repo && !info?.linked_repo;
  const hideServices =
    state === undefined ||
    state === Types.StackState.Unknown ||
    state === Types.StackState.Down;
  const hideLogs = hideServices || !specificLogs;
  const terminalDisabled =
    !specificTerminal ||
    containerTerminalsDisabled ||
    // All services are not running
    services?.every(
      (service) =>
        !service.container ||
        service.container.state !== Types.ContainerStateStatusEnum.Running,
    );

  const view =
    (_view === "Info" && hideInfo) ||
    (_view === "Services" && hideServices) ||
    (_view === "Terminals" && terminalDisabled) ||
    (_view === "Log" && hideLogs)
      ? "Config"
      : _view;

  const tabs = useMemo<TabNoContent[]>(
    () => [
      {
        value: "Config",
        icon: ICONS.Settings,
      },
      {
        value: "Info",
        hidden: hideInfo,
        icon: ICONS.Search,
      },
      {
        value: "Services",
        disabled: hideServices,
        icon: ICONS.Service,
      },
      {
        value: "Log",
        disabled: hideLogs,
        icon: ICONS.Log,
      },
      {
        value: "Terminals",
        disabled: terminalDisabled,
        hidden: !specificTerminal,
        icon: ICONS.Terminal,
      },
      {
        value: "Stats",
        icon: ICONS.Stats,
      },
    ],
    [
      hideInfo,
      hideServices,
      specificLogs,
      hideLogs,
      specificTerminal,
      terminalDisabled,
    ],
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
      type: "Stack",
      params: {
        stack: id,
      },
    }),
    [id],
  );

  let View = Selector;
  switch (view) {
    case "Config":
      View = <StackConfig id={id} titleOther={Selector} />;
      break;
    case "Info":
      View = <StackInfo id={id} titleOther={Selector} />;
      break;
    case "Services":
      View = <StackServices id={id} titleOther={Selector} />;
      break;
    case "Log":
      View = <StackLog id={id} titleOther={Selector} />;
      break;
    case "Terminals":
      View = (
        <TerminalSection
          target={target}
          services={services?.map((s) => s.service)}
          titleOther={Selector}
        />
      );
      break;
    case "Stats":
      View = <StackContainerStats id={id} titleOther={Selector} />;
      break;
  }

  return (
    <Tabs
      color={colorByIntention(
        stackStateIntention(
          state,
          info?.services &&
            !info.services.every((service) => !service.update_available),
        ),
      )}
      value={view}
    >
      {View}
    </Tabs>
  );
}
