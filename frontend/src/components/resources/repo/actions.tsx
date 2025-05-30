import { ConfirmButton } from "@components/util";
import { useExecute, useRead } from "@lib/hooks";
import {
  ArrowDownToDot,
  ArrowDownToLine,
  Ban,
  Hammer,
  Loader2,
} from "lucide-react";
import { useRepo } from ".";
import { Types } from "komodo_client";
import { useBuilder } from "../builder";

export const CloneRepo = ({ id }: { id: string }) => {
  const { mutate, isPending } = useExecute("CloneRepo");
  const cloning = useRead(
    "GetRepoActionState",
    { repo: id },
    { refetchInterval: 5000 }
  ).data?.cloning;
  const info = useRepo(id)?.info;
  if (!info?.server_id) return null;
  const hash = info?.latest_hash;
  const isCloned = (hash?.length || 0) > 0;
  const pending = isPending || cloning;
  return (
    <ConfirmButton
      title={isCloned ? "Reclone" : "Clone"}
      icon={
        pending ? (
          <Loader2 className="w-4 h-4 animate-spin" />
        ) : (
          <ArrowDownToLine className="w-4 h-4" />
        )
      }
      onClick={() => mutate({ repo: id })}
      disabled={pending}
      loading={pending}
    />
  );
};

export const PullRepo = ({ id }: { id: string }) => {
  const { mutate, isPending } = useExecute("PullRepo");
  const pulling = useRead(
    "GetRepoActionState",
    { repo: id },
    { refetchInterval: 5000 }
  ).data?.pulling;
  const info = useRepo(id)?.info;
  if (!info?.server_id) return null;
  const hash = info?.latest_hash;
  const isCloned = (hash?.length || 0) > 0;
  if (!isCloned) return null;
  const pending = isPending || pulling;
  return (
    <ConfirmButton
      title="Pull"
      icon={
        pending ? (
          <Loader2 className="w-4 h-4 animate-spin" />
        ) : (
          <ArrowDownToDot className="w-4 h-4" />
        )
      }
      onClick={() => mutate({ repo: id })}
      disabled={pending}
      loading={pending}
    />
  );
};

export const BuildRepo = ({ id }: { id: string }) => {
  const perms = useRead("GetPermission", {
    target: { type: "Repo", id },
  }).data;
  const building = useRead(
    "GetRepoActionState",
    { repo: id },
    { refetchInterval: 5000 }
  ).data?.building;
  const updates = useRead("ListUpdates", {
    query: {
      "target.type": "Repo",
      "target.id": id,
    },
  }).data;
  const { mutate: run_mutate, isPending: runPending } = useExecute("BuildRepo");
  const { mutate: cancel_mutate, isPending: cancelPending } =
    useExecute("CancelRepoBuild");

  const repo = useRepo(id);
  const builder = useBuilder(repo?.info.builder_id);
  const canCancel = builder?.info.builder_type !== "Server";

  // Don't show if builder not attached
  if (!builder) return null;

  // make sure hidden without perms.
  // not usually necessary, but this button also used in deployment actions.
  if (
    perms?.level !== Types.PermissionLevel.Execute &&
    perms?.level !== Types.PermissionLevel.Write
  )
    return null;

  // updates come in in descending order, so 'find' will find latest update matching operation
  const latestBuild = updates?.updates.find(
    (u) => u.operation === Types.Operation.BuildRepo
  );
  const latestCancel = updates?.updates.find(
    (u) => u.operation === Types.Operation.CancelRepoBuild
  );
  const cancelDisabled =
    !canCancel ||
    cancelPending ||
    (latestCancel && latestBuild
      ? latestCancel!.start_ts > latestBuild!.start_ts
      : false);

  if (building) {
    return (
      <ConfirmButton
        title="Cancel Build"
        variant="destructive"
        icon={<Ban className="h-4 w-4" />}
        onClick={() => cancel_mutate({ repo: id })}
        disabled={cancelDisabled}
      />
    );
  } else {
    return (
      <ConfirmButton
        title="Build"
        icon={
          runPending ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            <Hammer className="h-4 w-4" />
          )
        }
        onClick={() => run_mutate({ repo: id })}
        disabled={runPending}
      />
    );
  }
};
