import {
  useExecute,
  useInvalidate,
  useIsCancelling,
  useRead,
} from "@/lib/hooks";
import { useAction } from ".";
import ConfirmModalWithDisable from "@/components/confirm-modal-with-disable";
import { ICONS } from "@/lib/icons";
import { ConfirmButton } from "mogh_ui";
import { Types } from "komodo_client";
import { EXECUTION_ACTION_STATE_REQUERY_MS } from "@/lib/utils";

export function RunAction({ id }: { id: string }) {
  const invalidate = useInvalidate();
  const running =
    (useRead(
      "GetActionActionState",
      { action: id },
      { refetchInterval: 5_000 },
    ).data?.running ?? 0) > 0;
  const { mutateAsync: run, isPending: runPending } = useExecute("RunAction", {
    onSuccess: () =>
      setTimeout(
        () => invalidate(["GetActionActionState"]),
        EXECUTION_ACTION_STATE_REQUERY_MS,
      ),
  });
  const { mutateAsync: cancel, isPending: cancelPending } = useExecute(
    "CancelAction",
    {
      onSuccess: () =>
        setTimeout(
          () => invalidate(["GetActionActionState"]),
          EXECUTION_ACTION_STATE_REQUERY_MS,
        ),
    },
  );
  const action = useAction(id);
  const cancelling = useIsCancelling(
    { type: "Action", id },
    Types.Operation.RunAction,
    Types.Operation.CancelAction,
  );

  if (!action) {
    return null;
  }

  if (running) {
    return (
      <ConfirmButton
        variant="filled"
        color="red"
        icon={<ICONS.Cancel size="1rem" />}
        onClick={() => cancel({ action: id })}
        loading={cancelPending || cancelling}
      >
        Cancel Run
      </ConfirmButton>
    );
  } else {
    return (
      <ConfirmModalWithDisable
        icon={<ICONS.Run size="1rem" />}
        confirmText={action.name}
        onConfirm={() => run({ action: id, args: {} })}
        loading={runPending}
      >
        Run Action
      </ConfirmModalWithDisable>
    );
  }
}
