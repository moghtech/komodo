import { useInvalidate, useUser } from "@lib/hooks";
import { Types } from "komodo_client";
import { Button } from "@ui/button";
import { toast } from "@ui/use-toast";
import { atom, useAtom } from "jotai";
import { Circle } from "lucide-react";
import { ReactNode, useCallback, useEffect, useState } from "react";
import { cn } from "@lib/utils";
import { AUTH_TOKEN_STORAGE_KEY } from "@main";
import { ResourceComponents } from "@components/resources";
import { UsableResource } from "@types";
import { ResourceName } from "@components/resources/common";

const rws_atom = atom<WebSocket | null>(null);
const useWebsocket = () => useAtom(rws_atom);

const ws_connected = atom(false);
export const useWebsocketConnected = () => useAtom(ws_connected);

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

const on_message = (
  { data }: MessageEvent,
  invalidate: ReturnType<typeof useInvalidate>
) => {
  if (data == "LOGGED_IN") return console.info("logged in to ws");
  const update = JSON.parse(data) as Types.UpdateListItem;

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

    if (update.target.type === "ServerTemplate") {
      invalidate(
        ["ListServerTemplates"],
        ["ListFullServerTemplates"],
        ["GetServerTemplatesSummary"],
        ["GetServerTemplate"]
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

export const WebsocketProvider = ({
  url,
  children,
}: {
  url: string;
  children: ReactNode;
}) => {
  const user = useUser().data;
  const invalidate = useInvalidate();
  const [ws, set] = useWebsocket();
  const [connected, setConnected] = useWebsocketConnected();
  // don't care about value, just use to make sure value changes
  // to trigger connection useEffect
  const [reconnect, setReconnect] = useState(false);

  const on_message_fn = useCallback(
    (e: MessageEvent) => on_message(e, invalidate),
    [invalidate]
  );

  // Connection useEffect
  useEffect(() => {
    if (user && !connected) {
      const ws = make_websocket({
        url,
        on_open: () => setConnected(true),
        on_message: on_message_fn,
        on_close: () => {
          setConnected(false);
        },
      });
      set(ws);
    }
  }, [set, url, user, connected, reconnect]);

  useEffect(() => {
    // poll for CLOSED state.
    // trigger reconnect after stale page
    const interval = setInterval(() => {
      if (ws?.readyState === WebSocket.CLOSED) {
        setConnected(false);
        // toggle to make sure connection useEffect runs.
        // which could happen if connected is stuck in false state,
        // so setConnected(false) doesn't trigger reconnect
        setReconnect(!reconnect);
      }
    }, 3_000);

    return () => clearInterval(interval);
  }, []);

  return <>{children}</>;
};

export const WsStatusIndicator = () => {
  const [connected] = useWebsocketConnected();
  const onclick = () =>
    toast({
      title: connected ? "Websocket connected" : "Websocket disconnected",
    });

  return (
    <Button
      variant="ghost"
      onClick={onclick}
      size="icon"
      className="hidden lg:inline-flex"
    >
      <Circle
        className={cn(
          "w-4 h-4 stroke-none transition-colors",
          connected ? "fill-green-500" : "fill-red-500"
        )}
      />
    </Button>
  );
};

const make_websocket = ({
  url,
  on_open,
  on_message,
  on_close,
}: {
  url: string;
  on_open: () => void;
  on_message: (e: MessageEvent) => void;
  on_close: () => void;
}) => {
  const ws = new WebSocket(url);

  const _on_open = () => {
    const jwt = localStorage.getItem(AUTH_TOKEN_STORAGE_KEY);
    if (!ws || !jwt) return;
    const msg: Types.WsLoginMessage = { type: "Jwt", params: { jwt } };
    if (jwt && ws) ws.send(JSON.stringify(msg));
    on_open();
  };

  ws.addEventListener("open", _on_open);
  ws.addEventListener("message", on_message);
  ws.addEventListener("close", on_close);

  return ws;
};
