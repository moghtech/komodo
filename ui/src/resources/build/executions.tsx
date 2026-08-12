import { Types } from "komodo_client";
import {
  useExecute,
  useInvalidate,
  useIsCancelling,
  usePermissions,
  useRead,
} from "@/lib/hooks";
import { ConfirmButton } from "mogh_ui";
import { ICONS } from "@/lib/icons";
import { EXECUTION_ACTION_STATE_REQUERY_MS } from "@/lib/utils";

export function RunBuild({ id }: { id: string }) {
  const invalidate = useInvalidate();
  const { canExecute } = usePermissions({ type: "Build", id });
  const building = useRead(
    "GetBuildActionState",
    { build: id },
    { refetchInterval: 5_000 },
  ).data?.building;
  const { mutate: run, isPending: runPending } = useExecute("RunBuild", {
    onSuccess: () =>
      setTimeout(
        () => invalidate(["GetBuildActionState"]),
        EXECUTION_ACTION_STATE_REQUERY_MS,
      ),
  });
  const { mutate: cancel, isPending: cancelPending } = useExecute(
    "CancelBuild",
    {
      onSuccess: () =>
        setTimeout(
          () => invalidate(["GetBuildActionState"]),
          EXECUTION_ACTION_STATE_REQUERY_MS,
        ),
    },
  );
  const cancelling = useIsCancelling(
    { type: "Build", id },
    Types.Operation.RunBuild,
    Types.Operation.CancelBuild,
  );

  // make sure hidden without perms.
  // not usually necessary as execution area hidden without execute
  // on the resource, but this button is also used in deployment executions.
  if (!canExecute) return null;

  if (building) {
    return (
      <ConfirmButton
        variant="filled"
        color="red"
        icon={<ICONS.Cancel size="1rem" />}
        onClick={() => cancel({ build: id })}
        loading={cancelPending || cancelling}
      >
        Cancel Build
      </ConfirmButton>
    );
  } else {
    return (
      <ConfirmButton
        icon={<ICONS.Build size="1rem" />}
        onClick={() => run({ build: id })}
        loading={runPending}
      >
        Build
      </ConfirmButton>
    );
  }
}
