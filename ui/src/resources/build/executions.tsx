import { Types } from "komodo_client";
import { useExecute, usePermissions, useRead } from "@/lib/hooks";
import { useBuilder } from "../builder";
import { useBuild } from ".";
import ConfirmButton from "@/ui/confirm-button";
import { ICONS } from "@/theme/icons";

export function RunBuild({ id }: { id: string }) {
  const { canExecute } = usePermissions({ type: "Build", id });
  const building = useRead(
    "GetBuildActionState",
    { build: id },
    { refetchInterval: 5_000 },
  ).data?.building;
  const updates = useRead("ListUpdates", {
    query: {
      "target.type": "Build",
      "target.id": id,
    },
  }).data;
  const { mutate: runBuild, isPending: runPending } = useExecute("RunBuild");
  const { mutate: cancelBuild, isPending: cancelPending } =
    useExecute("CancelBuild");
  const build = useBuild(id);
  const builder = useBuilder(build?.info.builder_id);
  const canCancel = builder?.info.builder_type !== "Server";

  // make sure hidden without perms.
  // not usually necessary, but this button also used in deployment actions.
  if (!canExecute) return null;

  // updates come in in descending order, so 'find' will find latest update matching operation
  const latestBuild = updates?.updates.find(
    (u) => u.operation === Types.Operation.RunBuild,
  );
  const latestCancel = updates?.updates.find(
    (u) => u.operation === Types.Operation.CancelBuild,
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
        color="red"
        icon={<ICONS.Cancel size="1rem" />}
        onClick={() => cancelBuild({ build: id })}
        disabled={cancelDisabled}
      >
        Cancel Build
      </ConfirmButton>
    );
  } else {
    return (
      <ConfirmButton
        icon={<ICONS.Build size="1rem" />}
        loading={runPending}
        onClick={() => runBuild({ build: id })}
        disabled={runPending}
      >
        Build
      </ConfirmButton>
    );
  }
}
