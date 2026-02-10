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
import { useSwarm } from "../swarm";
import { useServer } from "../server";
import ResourceLink from "../link";
import { Group, Text } from "@mantine/core";
import { RunBuild } from "../build/executions";

export function useDeployment(id: string | undefined) {
  return useRead("ListDeployments", {}).data?.find((r) => r.id === id);
}

export function useFullDeployment(id: string) {
  return useRead("GetDeployment", { deployment: id }).data;
}

export const DeploymentComponents: RequiredResourceComponents<
  Types.DeploymentConfig,
  Types.DeploymentInfo,
  Types.DeploymentListItemInfo
> = {
  useList: () => useRead("ListDeployments", {}).data,
  useListItem: useDeployment,
  useFull: useFullDeployment,

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
    const deployment = useDeployment(id);
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
    let state = useDeployment(id)?.info.state;
    return (
      <StatusBadge text={state} intent={deploymentStateIntention(state)} />
    );
  },
  Info: {
    DeployTarget: ({ id }) => {
      const info = useDeployment(id)?.info;
      const swarm = useSwarm(info?.swarm_id);
      const server = useServer(info?.server_id);
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
    Image: ({ id }) => {
      const config = useFullDeployment(id)?.config;
      const info = useDeployment(id)?.info;
      return info?.build_id ? (
        <ResourceLink type="Build" id={info.build_id} />
      ) : (
        <Group gap="xs">
          <ICONS.Image size="1rem" />
          <Text>
            {info?.image.startsWith("sha256:")
              ? (
                  config?.image as Extract<
                    Types.DeploymentImage,
                    { type: "Image" }
                  >
                )?.params.image
              : info?.image.split("@")[0] || "N/A"}
          </Text>
        </Group>
      );
    },
  },

  Executions: {
    RunBuild: ({ id }) => {
      const build_id = useDeployment(id)?.info.build_id;
      if (!build_id) return null;
      return <RunBuild id={build_id} />;
    },
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
