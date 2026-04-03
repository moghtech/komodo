import { useExecute, usePermissions, useRead } from "@/lib/hooks";
import { useRepo } from ".";
import { useBuilder } from "../builder";
import { Types } from "komodo_client";
import ConfirmButton from "@/ui/confirm-button";
import { ICONS } from "@/lib/icons";

export function CloneRepo({ id }: { id: string }) {
  const { mutate, isPending } = useExecute("CloneRepo");
  const cloning = useRead(
    "GetRepoActionState",
    { repo: id },
    { refetchInterval: 5000 },
  ).data?.cloning;
  const info = useRepo(id)?.info;
  if (!info?.server_id) return null;
  const hash = info?.latest_hash;
  const isCloned = (hash?.length || 0) > 0;
  const pending = isPending || cloning;
  return (
    <ConfirmButton
      icon={<ICONS.CloneRepo size="1rem" />}
      onClick={() => mutate({ repo: id })}
      disabled={pending}
      loading={pending}
    >
      {isCloned ? "Reclone" : "Clone"}
    </ConfirmButton>
  );
}

export function PullRepo({ id }: { id: string }) {
  const { mutate, isPending } = useExecute("PullRepo");
  const pulling = useRead(
    "GetRepoActionState",
    { repo: id },
    { refetchInterval: 5000 },
  ).data?.pulling;
  const info = useRepo(id)?.info;
  if (!info?.server_id) return null;
  const hash = info?.latest_hash;
  const isCloned = (hash?.length || 0) > 0;
  if (!isCloned) return null;
  const pending = isPending || pulling;
  return (
    <ConfirmButton
      icon={<ICONS.PullRepo size="1rem" />}
      onClick={() => mutate({ repo: id })}
      disabled={pending}
      loading={pending}
    >
      Pull
    </ConfirmButton>
  );
}

export function BuildRepo({ id }: { id: string }) {
  const { canExecute } = usePermissions({ type: "Repo", id });
  const building = useRead(
    "GetRepoActionState",
    { repo: id },
    { refetchInterval: 5000 },
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
  if (!canExecute) return null;

  // updates come in in descending order, so 'find' will find latest update matching operation
  const latestBuild = updates?.updates.find(
    (u) => u.operation === Types.Operation.BuildRepo,
  );
  const latestCancel = updates?.updates.find(
    (u) => u.operation === Types.Operation.CancelRepoBuild,
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
        variant="filled"
        color="red"
        icon={<ICONS.Cancel size="1rem" />}
        onClick={() => cancel_mutate({ repo: id })}
        disabled={cancelDisabled}
        loading={cancelPending}
      >
        Cancel Build
      </ConfirmButton>
    );
  } else {
    return (
      <ConfirmButton
        icon={<ICONS.Build size="1rem" />}
        onClick={() => run_mutate({ repo: id })}
        disabled={runPending || building}
        loading={runPending || building}
      >
        Build
      </ConfirmButton>
    );
  }
}
