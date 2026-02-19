import { useExecute, useRead } from "@/lib/hooks";
import { useProcedure } from ".";
import { ICONS } from "@/theme/icons";
import ConfirmModalWithDisable from "@/components/confirm-modal-with-disable";

export function RunProcedure({ id }: { id: string }) {
  const running = useRead(
    "GetProcedureActionState",
    { procedure: id },
    { refetchInterval: 5000 },
  ).data?.running;
  const { mutateAsync: run, isPending } = useExecute("RunProcedure");
  const procedure = useProcedure(id);
  if (!procedure) return null;
  return (
    <ConfirmModalWithDisable
      confirmText={procedure.name}
      icon={<ICONS.Start size="1rem" />}
      onConfirm={() => run({ procedure: id })}
      disabled={running || isPending}
      loading={running || isPending}
    >
      {running ? "Running" : "Run Procedure"}
    </ConfirmModalWithDisable>
  );
}
