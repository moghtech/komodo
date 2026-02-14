import { useTerminalTargetPermissions, useWrite } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import ConfirmButton from "@/ui/confirm-button";
import { notifications } from "@mantine/notifications";
import { Types } from "komodo_client";

export default function DeleteTerminal({
  target,
  terminal,
  refetch,
}: {
  target: Types.TerminalTarget;
  terminal: string;
  refetch: () => void;
}) {
  const { canWrite } = useTerminalTargetPermissions(target);
  const { mutate, isPending } = useWrite("DeleteTerminal", {
    onSuccess: () => {
      refetch();
      notifications.show({ message: `Deleted Terminal '${terminal}'` });
    },
  });
  return (
    <ConfirmButton
      variant="filled"
      color="red"
      icon={<ICONS.Delete size="1rem" />}
      onClick={() => mutate({ target, terminal })}
      w={120}
      disabled={!canWrite}
      loading={isPending}
    >
      Delete
    </ConfirmButton>
  );
}
