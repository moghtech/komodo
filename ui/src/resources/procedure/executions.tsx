import {
  useExecute,
  useInvalidate,
  useIsCancelling,
  useRead,
} from "@/lib/hooks";
import { useProcedure } from ".";
import { ICONS } from "@/lib/icons";
import ConfirmModalWithDisable from "@/components/confirm-modal-with-disable";
import { ConfirmButton } from "mogh_ui";
import { Types } from "komodo_client";
import { EXECUTION_ACTION_STATE_REQUERY_MS } from "@/lib/utils";

export function RunProcedure({ id }: { id: string }) {
  const invalidate = useInvalidate();
  const running = useRead(
    "GetProcedureActionState",
    { procedure: id },
    { refetchInterval: 5_000 },
  ).data?.running;
  const { mutateAsync: run, isPending: runPending } = useExecute(
    "RunProcedure",
    {
      onSuccess: () =>
        setTimeout(
          () => invalidate(["GetProcedureActionState"]),
          EXECUTION_ACTION_STATE_REQUERY_MS,
        ),
    },
  );
  const { mutateAsync: cancel, isPending: cancelPending } = useExecute(
    "CancelProcedure",
    {
      onSuccess: () =>
        setTimeout(
          () => invalidate(["GetProcedureActionState"]),
          EXECUTION_ACTION_STATE_REQUERY_MS,
        ),
    },
  );
  const procedure = useProcedure(id);
  const cancelling = useIsCancelling(
    { type: "Procedure", id },
    Types.Operation.RunProcedure,
    Types.Operation.CancelProcedure,
  );

  if (!procedure) {
    return null;
  }

  if (running) {
    return (
      <ConfirmButton
        variant="filled"
        color="red"
        icon={<ICONS.Cancel size="1rem" />}
        onClick={() => cancel({ procedure: id })}
        loading={cancelPending || cancelling}
      >
        Cancel Run
      </ConfirmButton>
    );
  } else {
    return (
      <ConfirmModalWithDisable
        icon={<ICONS.Run size="1rem" />}
        confirmText={procedure.name}
        onConfirm={() => run({ procedure: id })}
        loading={runPending}
      >
        Run Procedure
      </ConfirmModalWithDisable>
    );
  }
}
