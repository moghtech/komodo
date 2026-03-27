import { useExecute } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { useNavigate } from "react-router-dom";
import ConfirmModalWithDisable from "@/components/confirm-modal-with-disable";

export type RemovableSwarmResourceType =
  | "Node"
  | "Stack"
  | "Service"
  | "Config"
  | "Secret";

export interface RemoveSwarmResourceProps {
  swarmId: string;
  type: RemovableSwarmResourceType;
  resourceId: string;
  resourceName?: string;
  disabled?: boolean;
}

export default function RemoveSwarmResource({
  swarmId,
  type,
  resourceId,
  resourceName,
  disabled,
}: RemoveSwarmResourceProps) {
  const nav = useNavigate();
  const { mutateAsync: remove, isPending } = useExecute(`RemoveSwarm${type}s`, {
    onSuccess: () => {
      nav("/swarms/" + swarmId);
    },
  });
  let key = `${type.toLowerCase()}s`;
  return (
    <ConfirmModalWithDisable
      confirmText={resourceName ?? resourceId}
      targetProps={{ variant: "filled", color: "red" }}
      icon={<ICONS.Delete size="1rem" />}
      disabled={disabled || isPending}
      loading={isPending}
      onConfirm={() =>
        remove({ swarm: swarmId, [key]: [resourceId], detach: false } as any)
      }
    >
      Remove
    </ConfirmModalWithDisable>
  );
}
