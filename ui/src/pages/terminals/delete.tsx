import { useTerminalTargetPermissions, useWrite } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import ConfirmButton from "@/ui/confirm-button";
import { ButtonProps } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { Types } from "komodo_client";
import { useNavigate } from "react-router-dom";

export default function DeleteTerminal({
  target,
  terminal,
  refetch,
  navTo,
  ...buttonProps
}: {
  target: Types.TerminalTarget;
  terminal: string;
  refetch?: () => void;
  navTo?: string;
} & ButtonProps) {
  const nav = useNavigate();
  const { canWrite } = useTerminalTargetPermissions(target);
  const { mutate, isPending } = useWrite("DeleteTerminal", {
    onSuccess: () => {
      refetch?.();
      notifications.show({
        message: `Deleted Terminal '${terminal}'`,
        color: "green",
      });
      navTo && nav(navTo);
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
      {...buttonProps}
    >
      Delete
    </ConfirmButton>
  );
}
