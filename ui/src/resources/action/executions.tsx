import { useExecute, useRead } from "@/lib/hooks";
import { useAction } from ".";
import ConfirmModalWithDisable from "@/components/confirm-modal-with-disable";
import { ICONS } from "@/lib/icons";
import { ConfirmButton } from "mogh_ui";

export function RunAction({ id }: { id: string }) {
  const running =
    (useRead("GetActionActionState", { action: id }, { refetchInterval: 5000 })
      .data?.running ?? 0) > 0;
  const { mutateAsync: run, isPending: runPending } = useExecute("RunAction");
  const { mutateAsync: cancel, isPending: cancelPending } =
    useExecute("CancelAction");
  const action = useAction(id);

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
        loading={cancelPending}
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
