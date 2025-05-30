import { ConfirmButton } from "@components/util";
import { useExecute, useRead } from "@lib/hooks";
import { Types } from "komodo_client";
import { Ban, Hammer } from "lucide-react";
import { useBuilder } from "../builder";

export const RunBuild = ({ id }: { id: string }) => {
  const perms = useRead("GetPermission", {
    target: { type: "Build", id },
  }).data;
  const building = useRead(
    "GetBuildActionState",
    { build: id },
    { refetchInterval: 5_000 }
  ).data?.building;
  const updates = useRead("ListUpdates", {
    query: {
      "target.type": "Build",
      "target.id": id,
    },
  }).data;
  const { mutate: run_mutate, isPending: runPending } = useExecute("RunBuild");
  const { mutate: cancel_mutate, isPending: cancelPending } =
    useExecute("CancelBuild");
  const build = useRead("ListBuilds", {}).data?.find((d) => d.id === id);
  const builder = useBuilder(build?.info.builder_id);
  const canCancel = builder?.info.builder_type !== "Server";

  // make sure hidden without perms.
  // not usually necessary, but this button also used in deployment actions.
  if (
    perms?.level !== Types.PermissionLevel.Execute &&
    perms?.level !== Types.PermissionLevel.Write
  )
    return null;

  // updates come in in descending order, so 'find' will find latest update matching operation
  const latestBuild = updates?.updates.find(
    (u) => u.operation === Types.Operation.RunBuild
  );
  const latestCancel = updates?.updates.find(
    (u) => u.operation === Types.Operation.CancelBuild
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
        onClick={() => cancel_mutate({ build: id })}
        disabled={cancelDisabled}
      />
    );
  } else {
    return (
      <ConfirmButton
        title="Build"
        icon={<Hammer className="h-4 w-4" />}
        loading={runPending}
        onClick={() => run_mutate({ build: id })}
        disabled={runPending}
      />
    );
  }
};
