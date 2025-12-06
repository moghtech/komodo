import { useServer } from "@components/resources/server";
import { ContainerTerminals } from "@components/terminal/container";
import { useLocalStorage, usePermissions } from "@lib/hooks";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "@ui/mobile-friendly-tabs";
import { Types } from "komodo_client";
import { useMemo } from "react";
import { StackServiceInspect } from "./inspect";
import { StackServiceLogs } from "./log";

type StackServiceTabsView = "Log" | "Inspect" | "Terminals";

export const StackServiceTabs = ({
  stack,
  service,
  container,
  swarm_service,
}: {
  stack: Types.StackListItem;
  service: string;
  container: Types.ContainerListItem | undefined;
  swarm_service: Types.SwarmServiceListItem | undefined;
}) => {
  const [_view, setView] = useLocalStorage<StackServiceTabsView>(
    `stack-${stack.id}-${service}-tabs-v2`,
    "Log"
  );
  const { specificLogs, specificInspect, specificTerminal } = usePermissions({
    type: "Stack",
    id: stack.id,
  });

  const down = !swarm_service && !container;

  const container_terminals_disabled =
    useServer(stack.info.server_id)?.info.container_terminals_disabled ?? false;
  const logDisabled = !specificLogs || down;
  const inspectDisabled = !specificInspect || down;
  const terminalDisabled =
    !specificTerminal ||
    container_terminals_disabled ||
    container?.state !== Types.ContainerStateStatusEnum.Running;
  const view =
    (inspectDisabled && _view === "Inspect") ||
    (terminalDisabled && _view === "Terminals")
      ? "Log"
      : _view;

  const tabs = useMemo<TabNoContent[]>(
    () => [
      {
        value: "Log",
        disabled: logDisabled,
      },
      {
        value: "Inspect",
        disabled: inspectDisabled,
      },
      {
        value: "Terminals",
        disabled: terminalDisabled,
        hidden: !container,
      },
    ],
    [logDisabled, inspectDisabled, terminalDisabled]
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
        stack: stack.id,
        service,
      },
    }),
    [stack.id, service]
  );

  switch (view) {
    case "Log":
      return (
        <StackServiceLogs
          id={stack.id}
          service={service}
          titleOther={Selector}
          disabled={logDisabled}
        />
      );
    case "Inspect":
      return (
        <StackServiceInspect
          id={stack.id}
          service={service}
          titleOther={Selector}
          useSwarm={!!swarm_service}
        />
      );
    case "Terminals":
      return <ContainerTerminals target={target} titleOther={Selector} />;
  }
};
