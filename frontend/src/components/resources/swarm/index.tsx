import { usePermissions, useRead } from "@lib/hooks";
import { RequiredResourceComponents } from "@types";
import {
  Check,
  Component,
  Diamond,
  FolderCode,
  KeyRound,
  ListTodo,
  Loader2,
  Settings,
  SquareStack,
} from "lucide-react";
import { DeleteResource, NewResource, ResourcePageHeader } from "../common";
import { SwarmTable } from "./table";
import {
  swarm_state_intention,
  stroke_color_class_by_intention,
  swarm_node_state_intention,
  swarm_task_state_intention,
} from "@lib/color";
import { cn, updateLogToHtml } from "@lib/utils";
import { Types } from "komodo_client";
import { CopyButton, DashboardPieChart } from "@components/util";
import { StatusBadge } from "@components/util";
import { GroupActions } from "@components/group-actions";
import { Tooltip, TooltipContent, TooltipTrigger } from "@ui/tooltip";
import { Card } from "@ui/card";
import { SwarmTabs } from "./tabs";
import { Link } from "react-router-dom";
import { ReactNode, useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@ui/dialog";
import { Button } from "@ui/button";
import { Input } from "@ui/input";

export const useSwarm = (id?: string) =>
  useRead("ListSwarms", {}, { refetchInterval: 10_000 }).data?.find(
    (d) => d.id === id
  );

export const useFullSwarm = (id: string) =>
  useRead("GetSwarm", { swarm: id }, { refetchInterval: 10_000 }).data;

const SwarmIcon = ({ id, size }: { id?: string; size: number }) => {
  const state = useSwarm(id)?.info.state;
  const color = stroke_color_class_by_intention(swarm_state_intention(state));
  return <Component className={cn(`w-${size} h-${size}`, state && color)} />;
};

export const SwarmComponents: RequiredResourceComponents = {
  list_item: (id) => useSwarm(id),
  resource_links: (resource) => (resource.config as Types.SwarmConfig).links,

  Description: () => <>Control and monitor docker swarms.</>,

  Dashboard: () => {
    const summary = useRead("GetSwarmsSummary", {}).data;
    return (
      <DashboardPieChart
        data={[
          { intention: "Good", value: summary?.healthy ?? 0, title: "Healthy" },
          {
            intention: "Critical",
            value: summary?.unhealthy ?? 0,
            title: "Unhealthy",
          },
          {
            intention: "Unknown",
            value: summary?.unknown ?? 0,
            title: "Unknown",
          },
        ]}
      />
    );
  },

  New: () => <NewResource type="Swarm" />,

  GroupActions: () => <GroupActions type="Swarm" actions={[]} />,

  Table: ({ resources }) => (
    <SwarmTable swarms={resources as Types.SwarmListItem[]} />
  ),

  Icon: ({ id }) => <SwarmIcon id={id} size={4} />,
  BigIcon: ({ id }) => <SwarmIcon id={id} size={8} />,

  State: ({ id }) => {
    const state = useSwarm(id)?.info.state;
    return <StatusBadge text={state} intent={swarm_state_intention(state)} />;
  },

  Info: {
    Join: ({ id }) => {
      const [open, setOpen] = useState(false);
      const { specificInspect } = usePermissions({ type: "Swarm", id });
      return (
        <Dialog open={open} onOpenChange={setOpen}>
          <DialogTrigger asChild>
            <Button variant="outline" disabled={!specificInspect}>
              Join
            </Button>
          </DialogTrigger>
          <DialogContent className="w-[95vw] max-w-[600px]">
            <JoinSwarmCommands id={id} close={() => setOpen(false)} />
          </DialogContent>
        </Dialog>
      );
    },
  },

  Status: {
    Err: ({ id }) => {
      const err = useSwarm(id)?.info.err;
      if (!err) return null;
      return (
        <Tooltip>
          <TooltipTrigger asChild>
            <Card className="px-3 py-2 bg-destructive/75 hover:bg-destructive transition-colors cursor-pointer">
              <div className="text-sm text-nowrap overflow-hidden overflow-ellipsis">
                Error
              </div>
            </Card>
          </TooltipTrigger>
          <TooltipContent className="w-fit max-w-[90vw] md:max-w-[60vw]">
            <pre
              dangerouslySetInnerHTML={{
                __html: updateLogToHtml(err),
              }}
            />
          </TooltipContent>
        </Tooltip>
      );
    },
  },

  Actions: {},

  Page: {},

  Config: SwarmTabs,

  DangerZone: ({ id }) => <DeleteResource type="Swarm" id={id} />,

  ResourcePageHeader: ({ id }) => {
    const swarm = useSwarm(id);

    return (
      <ResourcePageHeader
        intent={swarm_state_intention(swarm?.info.state)}
        icon={<SwarmIcon id={id} size={8} />}
        type="Swarm"
        id={id}
        resource={swarm}
        state={swarm?.info.state}
        status=""
      />
    );
  },
};

export type SwarmResourceType =
  | "Node"
  | "Service"
  | "Task"
  | "Secret"
  | "Config"
  | "Stack";

export const SWARM_ICONS: {
  [type in SwarmResourceType]: React.FC<{
    swarm_id?: string;
    resource_id?: string;
    size?: number;
    className?: string;
  }>;
} = {
  Node: ({ swarm_id, resource_id, size, className }) => {
    const state = useRead(
      "ListSwarmNodes",
      { swarm: swarm_id! },
      { enabled: !!swarm_id }
    ).data?.find((node) => resource_id && node.ID === resource_id)?.State;
    return (
      <Diamond
        className={cn(
          `w-${size} h-${size}`,
          stroke_color_class_by_intention(swarm_node_state_intention(state)),
          className
        )}
      />
    );
  },
  Stack: ({ swarm_id, resource_id, size, className }) => {
    const state = useRead(
      "ListSwarmStacks",
      { swarm: swarm_id! },
      { enabled: !!swarm_id }
    ).data?.find((stack) => resource_id && stack.Name === resource_id)?.State;
    return (
      <SquareStack
        className={cn(
          `w-${size} h-${size}`,
          stroke_color_class_by_intention(swarm_state_intention(state)),
          className
        )}
      />
    );
  },
  Service: ({ swarm_id, resource_id, size, className }) => {
    const state = useRead(
      "ListSwarmServices",
      { swarm: swarm_id! },
      { enabled: !!swarm_id }
    ).data?.find(
      (service) =>
        resource_id &&
        (service.ID === resource_id || service.Name === resource_id)
    )?.State;
    return (
      <FolderCode
        className={cn(
          `w-${size} h-${size}`,
          stroke_color_class_by_intention(swarm_state_intention(state)),
          className
        )}
      />
    );
  },
  Task: ({ swarm_id, resource_id, size, className }) => {
    const task = useRead(
      "ListSwarmTasks",
      { swarm: swarm_id! },
      { enabled: !!swarm_id }
    ).data?.find((task) => resource_id && task.ID === resource_id);
    return (
      <ListTodo
        className={cn(
          `w-${size} h-${size}`,
          stroke_color_class_by_intention(
            swarm_task_state_intention(task?.State, task?.DesiredState)
          ),
          className
        )}
      />
    );
  },
  Config: ({ swarm_id, resource_id, size, className }) => {
    const inUse = useRead(
      "ListSwarmConfigs",
      { swarm: swarm_id! },
      { enabled: !!swarm_id }
    ).data?.find(
      (config) =>
        resource_id &&
        (config.ID === resource_id || config.Name === resource_id)
    )?.InUse;
    return (
      <Settings
        className={cn(
          `w-${size} h-${size}`,
          stroke_color_class_by_intention(inUse ? "Good" : "Critical"),
          className
        )}
      />
    );
  },
  Secret: ({ swarm_id, resource_id, size, className }) => {
    const inUse = useRead(
      "ListSwarmSecrets",
      { swarm: swarm_id! },
      { enabled: !!swarm_id }
    ).data?.find(
      (secret) =>
        resource_id &&
        (secret.ID === resource_id || secret.Name === resource_id)
    )?.InUse;
    return (
      <KeyRound
        className={cn(
          `w-${size} h-${size}`,
          stroke_color_class_by_intention(inUse ? "Good" : "Critical"),
          className
        )}
      />
    );
  },
};

export const SwarmResourceLink = ({
  type,
  swarm_id,
  resource_id,
  name,
  extra,
}: {
  type: SwarmResourceType;
  swarm_id: string;
  resource_id: string | undefined;
  name: string | undefined;
  extra?: ReactNode;
}) => {
  const Icon = SWARM_ICONS[type];
  return (
    <Link
      to={`/swarms/${swarm_id}/swarm-${type.toLowerCase()}/${resource_id}`}
      className="flex items-center gap-2 text-sm hover:underline py-1"
    >
      <Icon swarm_id={swarm_id} resource_id={resource_id} size={4} />
      <div
        title={name}
        className="max-w-[250px] lg:max-w-[300px] overflow-hidden overflow-ellipsis break-words text-nowrap"
      >
        {name ?? "Unknown"}
      </div>
      {extra && <div className="no-underline">{extra}</div>}
    </Link>
  );
};

const JoinSwarmCommands = ({
  id,
  close,
}: {
  id: string;
  close: () => void;
}) => {
  const addr = useRead("ListSwarmNodes", { swarm: id }).data?.find(
    (node) => node.State === Types.NodeState.READY && node.ManagerAddr
  )?.ManagerAddr;
  const tokens = useRead("InspectSwarm", { swarm: id }).data?.JoinTokens;
  const managerCmd = `docker swarm join --token ${tokens?.Manager} ${addr}`;
  const workerCmd = `docker swarm join --token ${tokens?.Worker} ${addr}`;
  return (
    <>
      <DialogHeader>
        <DialogTitle>Join Swarm</DialogTitle>
        <DialogDescription>
          Copy a command below and run it on the target host to join it to the
          swarm.
        </DialogDescription>
      </DialogHeader>
      <div className="py-8 flex flex-col gap-4">
        {addr && tokens ? (
          <>
            <div className="flex items-center justify-between flex-wrap">
              As Manager
              <div className="flex items-center gap-4">
                <Input className="w-[250px]" value={managerCmd} disabled />
                <CopyButton content={managerCmd} />
              </div>
            </div>
            <div className="flex items-center justify-between flex-wrap">
              As Worker
              <div className="flex items-center gap-4">
                <Input className="w-[250px]" value={workerCmd} disabled />
                <CopyButton content={workerCmd} />
              </div>
            </div>
          </>
        ) : (
          <Loader2 className="w-16 h-16 animate-spin" />
        )}
      </div>
      <DialogFooter className="flex justify-end">
        <Button variant="secondary" className="gap-4" onClick={close}>
          Close <Check className="w-4" />
        </Button>
      </DialogFooter>
    </>
  );
};
