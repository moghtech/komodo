import { useInvalidate, komodo_client, useRead, useUser } from "@lib/hooks";
import { Types } from "komodo_client";
import { toast } from "@ui/use-toast";
import { atom, useAtom } from "jotai";
import { ReactNode, useCallback, useEffect, useRef } from "react";
import { ResourceComponents } from "@components/resources";
import { UsableResource } from "@types";
import { ResourceNameSimple } from "@components/resources/common";

const ws_atom = atom<{
  ws: WebSocket | undefined;
  connected: boolean;
  count: number;
}>({
  ws: undefined,
  connected: false,
  count: 0,
});

export const useWebsocketConnected = () => useAtom(ws_atom)[0].connected;

export const useWebsocketReconnect = () => {
  const [ws, set] = useAtom(ws_atom);

  return () => {
    if (ws.ws?.readyState === WebSocket.OPEN) {
      ws.ws?.close();
    }
    set((ws) => ({
      ws: undefined,
      connected: false,
      count: ws.count + 1,
    }));
  };
};

const onMessageHandlers: {
  [key: string]: (update: Types.UpdateListItem) => void;
} = {};

export const useWebsocketMessages = (
  key: string,
  handler: (update: Types.UpdateListItem) => void
) => {
  onMessageHandlers[key] = handler;
  useEffect(() => {
    // Clean up on unmount
    return () => {
      delete onMessageHandlers[key];
    };
  }, []);
};

export const WebsocketProvider = ({ children }: { children: ReactNode }) => {
  const user = useUser().data;
  const invalidate = useInvalidate();
  const [ws, setWs] = useAtom(ws_atom);
  const countRef = useRef<number>(ws.count);
  const reconnect = useWebsocketReconnect();
  const disable_reconnect = useRead("GetCoreInfo", {}).data
    ?.disable_websocket_reconnect;

  useEffect(() => {
    countRef.current = ws.count;
  }, [ws.count]);

  const on_update_fn = useCallback(
    (update: Types.UpdateListItem) => on_update(update, invalidate),
    [invalidate]
  );

  useEffect(() => {
    if (user && disable_reconnect !== undefined && ws.ws === undefined) {
      // make a copy of the count to not change.
      const count = ws.count;
      let timeout = -1;
      const socket = komodo_client().get_update_websocket({
        on_login: () => {
          console.info(count, "| Logged into Update websocket");
          setWs((ws) => ({ ...ws, connected: true }));
        },
        on_update: on_update_fn,
        on_close: () => {
          console.info(count, "| Update websocket connection closed");
          setWs((ws) => ({ ...ws, connected: false }));
          if (!disable_reconnect) {
            timeout = setTimeout(() => {
              if (countRef.current === count) {
                console.info(count, "| Automatically triggering reconnect");
                reconnect();
              }
            }, 5_000);
          }
        },
      });
      setWs((ws) => ({ ...ws, ws: socket }));
      return () => clearTimeout(timeout);
    }
  }, [user, disable_reconnect, ws.ws, ws.count]);

  return <>{children}</>;
};

const on_update = (
  update: Types.UpdateListItem,
  invalidate: ReturnType<typeof useInvalidate>
) => {
  const Components = ResourceComponents[update.target.type as UsableResource];
  const title = Components ? (
    <div className="flex items-center gap-2">
      <div>Update</div> -<div>{update.operation}</div> -
      <div>
        <ResourceNameSimple
          type={update.target.type as UsableResource}
          id={update.target.id}
        />
      </div>
      {!update.success && <div>FAILED</div>}
    </div>
  ) : (
    `${update.operation}${update.success ? "" : " - FAILED"}`
  );

  toast({ title: title as any });

  // Invalidate these every time
  invalidate(["ListUpdates"]);
  invalidate(["GetUpdate", { id: update.id }]);
  if (update.target.type === "Swarm") {
    invalidate(["GetSwarmActionState", { swarm: update.target.id }]);
  } else if (update.target.type === "Server") {
    invalidate(["GetServerActionState", { server: update.target.id }]);
  } else if (update.target.type === "Stack") {
    invalidate(["GetStackActionState", { stack: update.target.id }]);
  } else if (update.target.type === "Deployment") {
    invalidate(["GetDeploymentActionState", { deployment: update.target.id }]);
  } else if (update.target.type === "Build") {
    invalidate(["GetBuildActionState", { build: update.target.id }]);
  } else if (update.target.type === "Repo") {
    invalidate(["GetRepoActionState", { repo: update.target.id }]);
  } else if (update.target.type === "Procedure") {
    invalidate(["GetProcedureActionState", { procedure: update.target.id }]);
  } else if (update.target.type === "Action") {
    invalidate(["GetActionActionState", { action: update.target.id }]);
  } else if (update.target.type === "ResourceSync") {
    invalidate(["GetResourceSyncActionState", { sync: update.target.id }]);
  }

  // Invalidate lists for execution updates - this updates status
  if (update.operation === Types.Operation.RunBuild) {
    invalidate(["ListBuilds"]);
  } else if (
    [
      Types.Operation.CloneRepo,
      Types.Operation.PullRepo,
      Types.Operation.BuildRepo,
    ].includes(update.operation)
  ) {
    invalidate(["ListRepos"]);
  } else if (update.operation === Types.Operation.RunProcedure) {
    invalidate(["ListProcedures"]);
  } else if (update.operation === Types.Operation.RunAction) {
    invalidate(["ListActions"]);
  }

  // Do invalidations of these only if update is completed
  if (update.status === Types.UpdateStatus.Complete) {
    invalidate(["ListAlerts"]);

    // Invalidate docker infos
    if (["Server", "Deployment", "Stack"].includes(update.target.type)) {
      invalidate(
        ["ListDockerContainers"],
        ["InspectDockerContainer"],
        ["ListDockerNetworks"],
        ["InspectDockerNetwork"],
        ["ListDockerImages"],
        ["InspectDockerImage"],
        ["ListDockerVolumes"],
        ["InspectDockerVolume"],
        ["GetResourceMatchingContainer"]
      );
    }

    if (update.target.type === "Swarm") {
      invalidate(
        ["ListSwarms"],
        ["ListFullSwarms"],
        ["GetSwarmsSummary"],
        ["GetSwarm"],
        ["ListSwarmNodes"],
        ["InspectSwarmNode"],
        ["ListSwarmStacks"],
        ["InspectSwarmStack"],
        ["ListSwarmServices"],
        ["InspectSwarmService"],
        ["ListSwarmTasks"],
        ["InspectSwarmTask"],
        ["ListSwarmConfigs"],
        ["InspectSwarmConfig"],
        ["ListSwarmSecrets"],
        ["InspectSwarmSecret"]
      );
    }

    if (update.target.type === "Server") {
      invalidate(
        ["ListServers"],
        ["ListFullServers"],
        ["GetServersSummary"],
        ["GetServer"],
        ["GetServerState"],
        ["GetHistoricalServerStats"]
      );
    }

    if (update.target.type === "Stack") {
      invalidate(
        ["ListStacks"],
        ["ListFullStacks"],
        ["GetStacksSummary"],
        ["ListCommonStackExtraArgs"],
        ["ListComposeProjects"],
        ["ListDockerContainers"],
        ["ListDockerNetworks"],
        ["ListDockerImages"],
        ["GetStackLog", { stack: update.target.id }],
        ["SearchStackLog", { stack: update.target.id }],
        ["GetStack"],
        ["ListStackServices"],
        ["GetResourceMatchingContainer"]
      );
    }

    if (update.target.type === "Deployment") {
      invalidate(
        ["ListDeployments"],
        ["GetDeploymentsSummary"],
        ["ListDockerContainers"],
        ["ListDockerNetworks"],
        ["ListDockerImages"],
        ["GetDeployment"],
        ["GetDeploymentLog", { deployment: update.target.id }],
        ["SearchDeploymentLog", { deployment: update.target.id }],
        ["GetDeploymentContainer"],
        ["GetResourceMatchingContainer"]
      );
    }

    if (update.target.type === "Build") {
      invalidate(
        ["ListBuilds"],
        ["ListFullBuilds"],
        ["GetBuildsSummary"],
        ["GetBuildMonthlyStats"],
        ["GetBuild"],
        ["ListBuildVersions"]
      );
    }

    if (update.target.type === "Repo") {
      invalidate(
        ["ListRepos"],
        ["ListFullRepos"],
        ["GetReposSummary"],
        ["GetRepo"]
      );
    }

    if (update.target.type === "Procedure") {
      invalidate(
        ["ListSchedules"],
        ["ListProcedures"],
        ["ListFullProcedures"],
        ["GetProceduresSummary"],
        ["GetProcedure"]
      );
    }

    if (update.target.type === "Action") {
      invalidate(
        ["ListSchedules"],
        ["ListActions"],
        ["ListFullActions"],
        ["GetActionsSummary"],
        ["GetAction"]
      );
    }

    if (update.target.type === "ResourceSync") {
      invalidate(
        ["ListResourceSyncs"],
        ["ListFullResourceSyncs"],
        ["GetResourceSyncsSummary"],
        ["GetResourceSync"]
      );
    }

    if (update.target.type === "Builder") {
      invalidate(
        ["ListBuilders"],
        ["ListFullBuilders"],
        ["GetBuildersSummary"],
        ["GetBuilder"]
      );
    }

    if (update.target.type === "Alerter") {
      invalidate(
        ["ListAlerters"],
        ["ListFullAlerters"],
        ["GetAlertersSummary"],
        ["GetAlerter"]
      );
    }

    if (
      update.target.type === "System" &&
      update.operation.includes("Variable")
    ) {
      invalidate(["ListVariables"], ["GetVariable"]);
    }
  }

  // Run any attached handlers
  Object.values(onMessageHandlers).forEach((handler) => handler(update));
};
