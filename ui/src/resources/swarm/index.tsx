import {
  swarmStateIntention,
  hexColorByIntention,
  swarmNodeStateIntention,
  swarmTaskStateIntention,
} from "@/lib/color";
import { usePermissions, useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import SwarmTable from "./table";
import NewResource from "@/resources/new";
import SwarmTabs from "./tabs";
import { useDisclosure } from "@mantine/hooks";
import { Box, Button, HoverCard, Modal, Text } from "@mantine/core";
import JoinSwarmCommands from "./join-commands";
import { updateLogToHtml } from "@/lib/utils";
import ResourceHeader from "../header";
import BatchExecutions from "@/components/batch-executions";
import SwarmHeaderInfo from "./header-info";

export function useSwarm(id: string | undefined) {
  return useRead("ListSwarms", {}).data?.find((r) => r.id === id);
}

export function useFullSwarm(id: string) {
  return useRead("GetSwarm", { swarm: id }).data;
}

export const SwarmComponents: RequiredResourceComponents<
  Types.SwarmConfig,
  Types.SwarmInfo,
  Types.SwarmListItemInfo
> = {
  useList: () => useRead("ListSwarms", {}).data,
  useListItem: useSwarm,
  useFull: useFullSwarm,

  useResourceLinks: (swarm) => swarm?.config?.links,

  useDashboardSummaryData: () => {
    const summary = useRead("GetSwarmsSummary", {}).data;
    return [
      { intention: "Good", value: summary?.healthy ?? 0, title: "Healthy" },
      {
        intention: "Critical",
        value: (summary?.unhealthy ?? 0) + (summary?.down ?? 0),
        title: "Unhealthy",
      },
      {
        intention: "Unknown",
        value: summary?.unknown ?? 0,
        title: "Unknown",
      },
    ];
  },

  Description: () => <>Control and monitor docker swarms.</>,

  New: () => <NewResource type="Swarm" />,

  BatchExecutions: () => <BatchExecutions type="Swarm" executions={[]} />,

  Table: SwarmTable,

  Icon: ({ id, size = "1rem", noColor }) => {
    const state = useRead("ListSwarms", {}).data?.find((r) => r.id === id)?.info
      .state;
    const color = noColor
      ? undefined
      : state && hexColorByIntention(swarmStateIntention(state));
    return <ICONS.Swarm size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const swarm = useSwarm(id);
    return (
      <ResourceHeader
        type="Swarm"
        id={id}
        resource={swarm}
        intent={swarmStateIntention(swarm?.info.state)}
        icon={ICONS.Swarm}
        name={swarm?.name}
        state={swarm?.info.state}
      />
    );
  },

  State: ({ id }) => {
    let state = useSwarm(id)?.info.state;
    return <StatusBadge text={state} intent={swarmStateIntention(state)} />;
  },
  Info: {
    Join: ({ id }) => {
      const [opened, { open, close }] = useDisclosure();
      const { specificInspect } = usePermissions({ type: "Swarm", id });
      return (
        <Box>
          <Modal
            title={<Text fz="h3">Join Commands</Text>}
            opened={opened}
            onClose={close}
            size="auto"
          >
            <JoinSwarmCommands id={id} close={close} />
          </Modal>

          <Button
            onClick={open}
            leftSection={<ICONS.JoinSwarm size="1rem" />}
            disabled={!specificInspect}
          >
            Join
          </Button>
        </Box>
      );
    },
    Nodes: ({ id }) => {
      return (
        <SwarmHeaderInfo<Types.SwarmNodeListItem>
          swarmId={id}
          type="Node"
          resourceId={(node) => node.ID}
          resourceName={(node) => node.Hostname}
          resourceState={(node) => node.State}
          resourceStateIntent={(node) => swarmNodeStateIntention(node.State)}
        />
      );
    },
    Stacks: ({ id }) => {
      return (
        <SwarmHeaderInfo<Types.SwarmStackListItem>
          swarmId={id}
          type="Stack"
          resourceId={(stack) => stack.Name}
          resourceName={(stack) => stack.Name}
          resourceState={(stack) => stack.State}
          resourceStateIntent={(stack) => swarmStateIntention(stack.State)}
        />
      );
    },
    Services: ({ id }) => {
      return (
        <SwarmHeaderInfo<Types.SwarmServiceListItem>
          swarmId={id}
          type="Service"
          resourceId={(service) => service.ID}
          resourceName={(service) => service.Name}
          resourceState={(service) => service.State}
          resourceStateIntent={(service) => swarmStateIntention(service.State)}
        />
      );
    },
    Tasks: ({ id }) => {
      return (
        <SwarmHeaderInfo<Types.SwarmTaskListItem>
          swarmId={id}
          type="Task"
          resourceId={(task) => task.ID}
          resourceName={(task) => task.ID}
          resourceState={(task) => task.State}
          resourceStateIntent={(task) =>
            swarmTaskStateIntention(task.State, task.DesiredState)
          }
        />
      );
    },
    Configs: ({ id }) => {
      return (
        <SwarmHeaderInfo<Types.SwarmConfigListItem>
          swarmId={id}
          type="Config"
          resourceId={(config) => config.ID}
          resourceName={(config) => config.Name}
          resourceState={(config) => (config.InUse ? "In Use" : "Unused")}
          resourceStateIntent={(config) => (config.InUse ? "Good" : "Critical")}
        />
      );
    },
    Secrets: ({ id }) => {
      return (
        <SwarmHeaderInfo<Types.SwarmSecretListItem>
          swarmId={id}
          type="Secret"
          resourceId={(secret) => secret.ID}
          resourceName={(secret) => secret.Name}
          resourceState={(secret) => (secret.InUse ? "In Use" : "Unused")}
          resourceStateIntent={(secret) => (secret.InUse ? "Good" : "Critical")}
        />
      );
    },
    Err: ({ id }) => {
      const err = useSwarm(id)?.info.err;
      if (!err) return null;
      return (
        <Box>
          <HoverCard position="bottom-start">
            <HoverCard.Target>
              <Button
                variant="filled"
                color="red"
                leftSection={<ICONS.Alert size="1rem" />}
              >
                Error
              </Button>
            </HoverCard.Target>
            <HoverCard.Dropdown>
              <Text
                component="pre"
                fz="sm"
                mah={500}
                dangerouslySetInnerHTML={{
                  __html: updateLogToHtml(err),
                }}
                style={{ overflowY: "auto" }}
              />
            </HoverCard.Dropdown>
          </HoverCard>
        </Box>
      );
    },
  },

  Executions: {},

  Config: SwarmTabs,

  Page: {},
};
