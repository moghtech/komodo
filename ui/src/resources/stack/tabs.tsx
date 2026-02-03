import { useLocalStorage } from "@mantine/hooks";
import { StackComponents } from ".";
import { usePermissions, useRead } from "@/lib/hooks";
import { ServerComponents } from "../server";
import { Types } from "komodo_client";
import { useMemo } from "react";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "@/ui/mobile-friendly-tabs";
import StackConfig from "./config";
import StackInfo from "./info";
import { Tabs } from "@mantine/core";
import { ICONS } from "@/lib/icons";

type StackTabsView = "Config" | "Info" | "Services" | "Log" | "Terminals";

export default function StackTabs({ id }: { id: string }) {
  const [_view, setView] = useLocalStorage<StackTabsView>({
    key: "stack-tabs-v2",
    defaultValue: "Config",
  });
  const info = StackComponents.useListItem(id)?.info;
  const { specificLogs, specificTerminal } = usePermissions({
    type: "Stack",
    id,
  });

  const services = useRead("ListStackServices", { stack: id }).data;

  const container_terminals_disabled =
    ServerComponents.useListItem(info?.server_id)?.info
      .container_terminals_disabled ?? false;

  const state = info?.state;
  const hideInfo = !info?.files_on_host && !info?.repo && !info?.linked_repo;
  const hideServices =
    state === undefined ||
    state === Types.StackState.Unknown ||
    state === Types.StackState.Down;
  const hideLogs = hideServices || !specificLogs;
  const terminalDisabled =
    !specificTerminal ||
    container_terminals_disabled ||
    // All services are not running
    services?.every(
      (service) =>
        !service.container ||
        service.container.state !== Types.ContainerStateStatusEnum.Running,
    );

  const view =
    (_view === "Info" && hideInfo) ||
    (_view === "Services" && hideServices) ||
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
      },
      {
        value: "Log",
        disabled: hideLogs,
      },
      {
        value: "Terminals",
        disabled: terminalDisabled,
        hidden: !specificTerminal,
      },
    ],
    [hideInfo, hideServices, specificLogs, hideLogs],
  );

  const Selector = (
    <MobileFriendlyTabsSelector
      tabs={tabs}
      value={view}
      onValueChange={setView as any}
      tabProps={{ w: 110 }}
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

  let View = null;
  switch (view) {
    case "Config":
      View = <StackConfig id={id} titleOther={Selector} />;
      break;
    case "Info":
      View = <StackInfo id={id} titleOther={Selector} />;
      break;
    case "Services":
      View = Selector;
      break;
    // return <StackServices id={id} titleOther={Selector} />;
    case "Log":
      View = Selector;
      break;
    // return <StackLogs id={id} titleOther={Selector} />;
    case "Terminals":
      View = Selector;
      break;
    // return (
    //   <ContainerTerminals
    //     services={services?.map((service) => service.service) ?? []}
    //     target={target}
    //     titleOther={Selector}
    //   />
    // );
  }

  return (
    <Tabs variant="pills" value={view}>
      {View}
    </Tabs>
  );
}
