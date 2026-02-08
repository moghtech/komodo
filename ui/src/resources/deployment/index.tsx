import { deploymentStateIntention, hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import EntityHeader from "@/ui/entity-header";
import DeploymentTable from "./table";
import NewResource from "@/resources/new";
import DeploymentTabs from "./tabs";
import DeleteResource from "../delete";
import {
  DeployDeployment,
  DestroyDeployment,
  PauseUnpauseDeployment,
  PullDeployment,
  RestartDeployment,
  StartStopDeployment,
} from "./executions";
import { SwarmComponents } from "../swarm";
import { ServerComponents } from "../server";
import ResourceLink from "../link";
import { Group, Text } from "@mantine/core";

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
      <EntityHeader
        intent={deploymentStateIntention(deployment?.info.state)}
        icon={ICONS.Deployment}
        name={deployment?.name}
        state={deployment?.info.state}
        status={deployment?.info.status}
        action={<DeleteResource type="Deployment" id={id} />}
      />
    );
  },

  State: ({ id }) => {
    let state = DeploymentComponents.useListItem(id)?.info.state;
    return (
      <StatusBadge text={state} intent={deploymentStateIntention(state)} />
    );
  },
  Info: {
    DeployTarget: ({ id }) => {
      const info = DeploymentComponents.useListItem(id)?.info;
      const swarm = SwarmComponents.useListItem(info?.swarm_id);
      const server = ServerComponents.useListItem(info?.server_id);
      return swarm?.id ? (
        <ResourceLink type="Swarm" id={swarm?.id} />
      ) : server?.id ? (
        <ResourceLink type="Server" id={server?.id} />
      ) : (
        <Group gap="xs">
          <ICONS.Server size="1rem" />
          <Text>Unknown</Text>
        </Group>
      );
    },
  },

  Executions: {
    DeployDeployment,
    PullDeployment,
    RestartDeployment,
    PauseUnpauseDeployment,
    StartStopDeployment,
    DestroyDeployment,
  },

  Config: DeploymentTabs,

  Page: {},
};
