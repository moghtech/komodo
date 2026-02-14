import { useExecute, useRead } from "@/lib/hooks";
import { useProcedure } from ".";
import ConfirmModal from "@/ui/confirm-modal";
import { ICONS } from "@/theme/icons";

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
    <ConfirmModal
      confirmText={procedure.name}
      icon={<ICONS.Start size="1rem" />}
      onConfirm={() => run({ procedure: id })}
      disabled={running || isPending}
      loading={running || isPending}
    >
      {running ? "Running" : "Run Procedure"}
    </ConfirmModal>
  );
}
