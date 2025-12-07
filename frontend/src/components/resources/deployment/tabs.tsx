import { Types } from "komodo_client";
import { useDeployment } from ".";
import { useLocalStorage, usePermissions } from "@lib/hooks";
import { useServer } from "../server";
import { useMemo } from "react";
import {
  MobileFriendlyTabsSelector,
  TabNoContent,
} from "@ui/mobile-friendly-tabs";
import { DeploymentConfig } from "./config";
import { DeploymentLogs } from "./log";
import { DeploymentInspect } from "./inspect";
import { ContainerTerminals } from "@components/terminal/container";

export const DeploymentTabs = ({ id }: { id: string }) => {
  const deployment = useDeployment(id);
  if (!deployment) return null;
  return <DeploymentTabsInner deployment={deployment} />;
};

type DeploymentTabsView = "Config" | "Log" | "Inspect" | "Terminals";

const DeploymentTabsInner = ({
  deployment,
}: {
  deployment: Types.DeploymentListItem;
}) => {
  const [_view, setView] = useLocalStorage<DeploymentTabsView>(
    "deployment-tabs-v1",
    "Config"
  );
  const { specificLogs, specificInspect, specificTerminal } = usePermissions({
    type: "Deployment",
    id: deployment.id,
  });
  const container_terminals_disabled =
    useServer(deployment.info.server_id)?.info.container_terminals_disabled ??
    false;
  const state = deployment.info.state;
  const downOrUnknown =
    state === undefined ||
    state === Types.DeploymentState.Unknown ||
    state === Types.DeploymentState.NotDeployed;
  const logsDisabled = !specificLogs || downOrUnknown;
  const inspectDisabled = !specificInspect || downOrUnknown;
  const terminalDisabled =
    !specificTerminal ||
    container_terminals_disabled ||
    state !== Types.DeploymentState.Running;
  const view =
    (logsDisabled && _view === "Log") ||
    (inspectDisabled && _view === "Inspect") ||
    (terminalDisabled && _view === "Terminals")
      ? "Config"
      : _view;

  const tabs = useMemo<TabNoContent<DeploymentTabsView>[]>(
    () => [
      {
        value: "Config",
      },
      {
        value: "Log",
        disabled: logsDisabled,
      },
      {
        value: "Inspect",
        disabled: inspectDisabled,
      },
      {
        value: "Terminals",
        disabled: terminalDisabled,
        hidden: !!deployment.info.swarm_id,
      },
    ],
    [logsDisabled, inspectDisabled, terminalDisabled, deployment.info.swarm_id]
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
      type: "Deployment",
      params: {
        deployment: deployment.id,
      },
    }),
    [deployment.id]
  );

  switch (view) {
    case "Config":
      return <DeploymentConfig id={deployment.id} titleOther={Selector} />;
    case "Log":
      return <DeploymentLogs id={deployment.id} titleOther={Selector} />;
    case "Inspect":
      return (
        <DeploymentInspect
          id={deployment.id}
          titleOther={Selector}
          useSwarm={!!deployment.info.swarm_id}
        />
      );
    case "Terminals":
      return <ContainerTerminals target={target} titleOther={Selector} />;
  }
};
