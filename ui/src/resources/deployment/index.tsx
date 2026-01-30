import { deploymentStateIntention, hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import ResourceHeader from "@/components/resource-header";
import DeploymentTable from "./table";
import NewResource from "@/resources/new";

export const DeploymentComponents: RequiredResourceComponents<
  Types.DeploymentConfig,
  Types.DeploymentInfo,
  Types.DeploymentListItemInfo
> = {
  useList: () => useRead("ListDeployments", {}).data,
  useListItem: (id) => DeploymentComponents.useList()?.find((r) => r.id === id),

  useFull: (id) => useRead("GetDeployment", { deployment: id }).data,

  useResourceLinks: (deployment) => deployment?.config?.links,

  useDashboardSummaryData: () => {
    const summary = useRead("GetDeploymentsSummary", {}).data;
    const all = [
      summary?.running ?? 0,
      summary?.stopped ?? 0,
      summary?.unhealthy ?? 0,
      summary?.unknown ?? 0,
    ];
    const [running, stopped, unhealthy, unknown] = all;
    return [
      all.every((item) => item === 0) && {
        title: "Not Deployed",
        intention: "Neutral",
        value: summary?.not_deployed ?? 0,
      },
      { intention: "Good", value: running, title: "Running" },
      {
        title: "Stopped",
        intention: "Warning",
        value: stopped,
      },
      {
        title: "Unhealthy",
        intention: "Critical",
        value: unhealthy,
      },
      {
        title: "Unknown",
        intention: "Unknown",
        value: unknown,
      },
    ];
  },

  Description: () => (
    <>Connect deployments for alerting, building, and deploying.</>
  ),

  New: () => <NewResource type="Deployment" />,

  GroupExecutions: () => <></>,

  Table: DeploymentTable,

  Icon: ({ id, size = "1rem" }) => {
    const state = useRead("ListDeployments", {}).data?.find((r) => r.id === id)
      ?.info.state;
    const color = state && hexColorByIntention(deploymentStateIntention(state));
    return <ICONS.Deployment size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const deployment = DeploymentComponents.useListItem(id);
    return (
      <ResourceHeader
        intent={deploymentStateIntention(deployment?.info.state)}
        icon={<DeploymentComponents.Icon id={id} size="2rem" />}
        name={deployment?.name}
        state={deployment?.info.state}
        status={deployment?.info.status}
      />
    );
  },

  State: ({ id }) => {
    let state = DeploymentComponents.useListItem(id)?.info.state;
    return (
      <StatusBadge text={state} intent={deploymentStateIntention(state)} />
    );
  },
  Status: {},
  Info: {},

  Executions: {},

  Config: () => <>CONFIG</>,
  DangerZone: ({ id }) => <></>,

  Page: {},
};
