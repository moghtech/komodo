import { usePermissions, useRead } from "@/lib/hooks";
import { useLocalStorage } from "@mantine/hooks";
import { useServer } from "../server";
import { useDeployment } from ".";
import { Types } from "komodo_client";
import { ReactNode, useMemo } from "react";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "mogh_ui";
import { ICONS } from "@/lib/icons";
import { Tabs } from "@mantine/core";
import { colorByIntention, deploymentStateIntention } from "@/lib/color";
import DeploymentConfig from "./config";
import LogSection from "@/components/log-section";
import TerminalSection from "@/components/terminal/section";
import { MonacoEditor } from "@/components/monaco";
import { Section } from "mogh_ui";

type DeploymentTabsView = "Config" | "Tasks" | "Log" | "Inspect" | "Terminals";

export default function DeploymentTabs({ id }: { id: string }) {
  const [_view, setView] = useLocalStorage<DeploymentTabsView>({
    key: "deployment-tabs-v1",
    defaultValue: "Config",
  });
  const info = useDeployment(id)?.info;
  const { specificLogs, specificInspect, specificTerminal } = usePermissions({
    type: "Deployment",
    id,
  });
  const containerTerminalsDisabled =
    useServer(info?.server_id)?.info.container_terminals_disabled ?? false;
  const state = info?.state;

  const downOrUnknown =
    state === undefined ||
    state === Types.DeploymentState.Unknown ||
    state === Types.DeploymentState.NotDeployed;

  const logsDisabled = !specificLogs || downOrUnknown;
  const inspectDisabled = !specificInspect || downOrUnknown;

  const terminalDisabled =
    !specificTerminal ||
    containerTerminalsDisabled ||
    state !== Types.DeploymentState.Running;

  const view =
    (logsDisabled && _view === "Log") ||
    (downOrUnknown && _view === "Tasks") ||
    (inspectDisabled && _view === "Inspect") ||
    (terminalDisabled && _view === "Terminals")
      ? "Config"
      : _view;

  const tabs = useMemo<TabNoContent[]>(
    () => [
      {
        value: "Config",
        icon: ICONS.Config,
      },
      {
        value: "Tasks",
        icon: ICONS.SwarmTask,
        disabled: downOrUnknown,
        hidden: !info?.swarm_id,
      },
      {
        value: "Log",
        icon: ICONS.Log,
        disabled: logsDisabled,
      },
      {
        value: "Inspect",
        icon: ICONS.Inspect,
        disabled: inspectDisabled,
      },
      {
        value: "Terminals",
        icon: ICONS.Terminal,
        disabled: terminalDisabled,
        hidden: !!info?.swarm_id,
      },
    ],
    [logsDisabled, inspectDisabled, terminalDisabled, info?.swarm_id],
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
      type: "Deployment",
      params: {
        deployment: id,
      },
    }),
    [id],
  );

  let View = Selector;
  switch (view) {
    case "Config":
      View = <DeploymentConfig id={id} titleOther={Selector} />;
      break;
    case "Tasks":
      break;
    case "Log":
      View = (
        <LogSection
          target={{ type: "Deployment", deploymentId: id }}
          titleOther={Selector}
        />
      );
      break;
    case "Inspect":
      View = <InspectDeploymentContainer id={id} titleOther={Selector} />;
      break;
    case "Terminals":
      View = <TerminalSection target={target} titleOther={Selector} />;
      break;
  }

  return (
    <Tabs
      color={colorByIntention(
        deploymentStateIntention(state, info?.update_available),
      )}
      value={view}
    >
      {View}
    </Tabs>
  );
}

function InspectDeploymentContainer({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const inspect = useRead("InspectDeploymentContainer", {
    deployment: id,
  }).data;
  return (
    <Section titleOther={titleOther}>
      <MonacoEditor
        value={inspect ? JSON.stringify(inspect, null, 2) : "NO DATA"}
        language="json"
        readOnly
      />
    </Section>
  );
}
