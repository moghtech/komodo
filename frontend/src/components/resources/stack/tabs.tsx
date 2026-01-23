import { useLocalStorage, usePermissions, useRead } from "@lib/hooks";
import { useStack } from ".";
import { Types } from "komodo_client";
import { useMemo } from "react";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "@ui/mobile-friendly-tabs";
import { StackInfo } from "./info";
import { StackServices } from "./services";
import { StackLogs } from "./log";
import { StackConfig } from "./config";
import { useServer } from "../server";
import { ContainerTerminals } from "@components/terminal/container";

type StackTabsView = "Config" | "Info" | "Services" | "Log" | "Terminals";

export const StackTabs = ({ id }: { id: string }) => {
  const [_view, setView] = useLocalStorage<StackTabsView>(
    "stack-tabs-v2",
    "Config",
  );
  const info = useStack(id)?.info;
  const { specificLogs, specificTerminal } = usePermissions({
    type: "Stack",
    id,
  });

  const services = useRead("ListStackServices", { stack: id }).data;

  const container_terminals_disabled =
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

  const tabs = useMemo<TabNoContent<StackTabsView>[]>(
    () => [
      {
        value: "Config",
      },
      {
        value: "Info",
        hidden: hideInfo,
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
      tabsTriggerClassname="w-[110px]"
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

  switch (view) {
    case "Config":
      return <StackConfig id={id} titleOther={Selector} />;
    case "Info":
      return <StackInfo id={id} titleOther={Selector} />;
    case "Services":
      return <StackServices id={id} titleOther={Selector} />;
    case "Log":
      return <StackLogs id={id} titleOther={Selector} />;
    case "Terminals":
      return (
        <ContainerTerminals
          services={services?.map((service) => service.service) ?? []}
          target={target}
          titleOther={Selector}
        />
      );
  }
};
