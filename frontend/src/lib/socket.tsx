import { komodo_client, useInvalidate, useRead, useUser } from "@lib/hooks";
import { CancelToken, Types } from "komodo_client";
import { Button } from "@ui/button";
import { toast } from "@ui/use-toast";
import { atom, useAtom } from "jotai";
import { Circle, Loader2 } from "lucide-react";
import { ReactNode, useCallback, useEffect, useState } from "react";
import { cn } from "@lib/utils";
import { ResourceComponents } from "@components/resources";
import { UsableResource } from "@types";
import { ResourceName } from "@components/resources/common";

const ws_connected = atom(false);
export const useWebsocketConnected = () => useAtom(ws_connected);

const ws_cancel = atom(new CancelToken());
const useWebsocketCancel = () => useAtom(ws_cancel)[0];

const useWebsocketReconnect = () => {
  const [cancel, set] = useAtom(ws_cancel);
  return () => {
    cancel.cancel();
    set(new CancelToken());
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
  const cancel = useWebsocketCancel();
  const [_, setConnected] = useWebsocketConnected();
  const disable_reconnect = useRead("GetCoreInfo", {}).data
    ?.disable_websocket_reconnect;

  const on_update_fn = useCallback(
    (update: Types.UpdateListItem) => on_update(update, invalidate),
    [invalidate]
  );

  useEffect(() => {
    if (user && disable_reconnect !== undefined) {
      komodo_client().subscribe_to_update_websocket({
        on_login: () => {
          setConnected(true);
          console.info("Logged into Update websocket");
        },
        on_update: on_update_fn,
        on_close: () => {
          setConnected(false);
          console.info("Update websocket connection closed");
        },
        retry: !disable_reconnect,
        cancel,
      });
    }
  }, [user, disable_reconnect, cancel]);

  return <>{children}</>;
};

export const WsStatusIndicator = () => {
  const [refreshing, setRefreshing] = useState(false);
  const [connected] = useWebsocketConnected();
  const reconnect = useWebsocketReconnect();
  const onclick = () => {
    setRefreshing(true);
    setTimeout(() => setRefreshing(false), 500);
    reconnect();
    toast({
      title: connected
        ? "Triggered websocket reconnect"
        : "Triggered websocket connect",
    });
  };

  return (
    <Button
      variant="ghost"
      onClick={onclick}
      size="icon"
      className="hidden lg:inline-flex"
    >
      {refreshing ? (
        <Loader2 className="w-4 h-4 animate-spin" />
      ) : (
        <Circle
          className={cn(
            "w-4 h-4 stroke-none transition-colors",
            connected ? "fill-green-500" : "fill-red-500"
          )}
        />
      )}
    </Button>
  );
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
        <ResourceName
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
  if (update.target.type === "Deployment") {
    invalidate(["GetDeploymentActionState", { deployment: update.target.id }]);
  } else if (update.target.type === "Stack") {
    invalidate(["GetStackActionState", { stack: update.target.id }]);
  } else if (update.target.type === "Server") {
    invalidate(["GetServerActionState", { server: update.target.id }]);
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

  // Invalidate lists for execution updates - update status
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
        ["ListProcedures"],
        ["ListFullProcedures"],
        ["GetProceduresSummary"],
        ["GetProcedure"]
      );
    }

    if (update.target.type === "Action") {
      invalidate(
        ["ListActions"],
        ["ListFullActions"],
        ["GetActionsSummary"],
        ["GetAction"]
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

    if (update.target.type === "ResourceSync") {
      invalidate(
        ["ListResourceSyncs"],
        ["ListFullResourceSyncs"],
        ["GetResourceSyncsSummary"],
        ["GetResourceSync"]
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
