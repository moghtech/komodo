import LogSection from "@/components/log-section";
import TerminalSection from "@/components/terminal/section";
import { colorByIntention, ColorIntention } from "@/lib/color";
import { usePermissions } from "@/lib/hooks";
import { useServer } from "@/resources/server";
import { ICONS } from "@/theme/icons";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "@/ui/mobile-friendly-tabs";
import { Tabs } from "@mantine/core";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";
import { useMemo, useState } from "react";
import StackServiceInspect from "./inspect";
import SwarmServiceTasksSection from "../swarm/service/tasks";

export type StackServiceTabsView = "Tasks" | "Log" | "Inspect" | "Terminals";

export interface StackServiceTabsProps {
  stack: Types.StackListItem;
  service: string;
  container: Types.ContainerListItem | undefined;
  swarmService: Types.SwarmServiceListItem | undefined;
  intention: ColorIntention;
}

export default function StackServiceTabs({
  stack,
  service,
  container,
  swarmService,
  intention,
}: StackServiceTabsProps) {
  const [_view, setView] = useLocalStorage<StackServiceTabsView>({
    key: `stack-${stack.id}-${service}-tabs-v2`,
    defaultValue: "Log",
  });
  const { specificLogs, specificInspect, specificTerminal } = usePermissions({
    type: "Stack",
    id: stack.id,
  });

  const down = !swarmService && !container;

  const containerTerminalsDisabled =
    useServer(stack.info.server_id)?.info.container_terminals_disabled ?? false;

  const logDisabled = !specificLogs || down;
  const inspectDisabled = !specificInspect || down;

  const terminalDisabled =
    !specificTerminal ||
    containerTerminalsDisabled ||
    container?.state !== Types.ContainerStateStatusEnum.Running;

  const view =
    (!stack.info.swarm_id && _view === "Tasks") ||
    (inspectDisabled && _view === "Inspect") ||
    (terminalDisabled && _view === "Terminals")
      ? "Log"
      : _view;

  const tabs = useMemo<TabNoContent[]>(
    () => [
      {
        value: "Tasks",
        hidden: !swarmService,
        icon: ICONS.SwarmTask,
      },
      {
        value: "Log",
        disabled: logDisabled,
        icon: ICONS.Log,
      },
      {
        value: "Inspect",
        disabled: inspectDisabled,
        icon: ICONS.Inspect,
      },
      {
        value: "Terminals",
        disabled: terminalDisabled,
        hidden: !container,
        icon: ICONS.Terminal,
      },
    ],
    [!swarmService, logDisabled, inspectDisabled, terminalDisabled],
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
        stack: stack.id,
        service,
      },
    }),
    [stack.id, service],
  );

  const _search = useState("");

  let View = Selector;
  switch (view) {
    case "Tasks":
      View = (
        <SwarmServiceTasksSection
          id={stack.info.swarm_id}
          serviceId={swarmService?.ID}
          titleOther={Selector}
          _search={_search}
        />
      );
      break;
    case "Log":
      View = (
        <LogSection
          target={{ type: "Stack", stackId: stack.id, services: [service] }}
          titleOther={Selector}
          disabled={logDisabled}
        />
      );
      break;
    case "Inspect":
      View = (
        <StackServiceInspect
          stackId={stack.id}
          service={service}
          useSwarm={!!swarmService}
          titleOther={Selector}
        />
      );
      break;
    case "Terminals":
      View = <TerminalSection target={target} titleOther={Selector} />;
      break;
  }

  return (
    <Tabs color={colorByIntention(intention)} value={view}>
      {View}
    </Tabs>
  );
}
