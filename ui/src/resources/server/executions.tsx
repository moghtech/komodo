import { useExecute, usePermissions, useRead } from "@/lib/hooks";
import { useServer } from ".";
import ConfirmButton from "@/ui/confirm-button";
import { ICONS } from "@/lib/icons";
import ConfirmModalWithDisable from "@/components/confirm-modal-with-disable";

export const Prune = ({
  serverId,
  type,
}: {
  serverId: string;
  type: "Containers" | "Networks" | "Images" | "Volumes" | "Buildx" | "System";
}) => {
  const server = useServer(serverId);
  const { mutateAsync: prune, isPending } = useExecute(`Prune${type}`);
  const action_state = useRead(
    "GetServerActionState",
    { server: serverId },
    { refetchInterval: 5000 },
  ).data;
  const { canExecute } = usePermissions({ type: "Server", id: serverId });

  if (!server) return;

  const pruningKey =
    type === "Containers"
      ? "pruning_containers"
      : type === "Images"
        ? "pruning_images"
        : type === "Networks"
          ? "pruning_networks"
          : type === "Volumes"
            ? "pruning_volumes"
            : type === "Buildx"
              ? "pruning_buildx"
              : type === "System"
                ? "pruning_system"
                : "";

  const pending =
    isPending ||
    (pruningKey && action_state?.[pruningKey]
      ? action_state?.[pruningKey]
      : undefined);

  if (type === "Images" || type === "Networks" || type === "Buildx") {
    return (
      <ConfirmButton
        icon={<ICONS.Prune size="1rem" />}
        onClick={() => prune({ server: serverId })}
        loading={pending}
        disabled={!canExecute || pending}
      >
        {`Prune ${type}`}
      </ConfirmButton>
    );
  } else {
    return (
      <ConfirmModalWithDisable
        confirmText={server?.name}
        icon={<ICONS.Prune size="1rem" />}
        onConfirm={() => prune({ server: serverId })}
        loading={pending}
        disabled={!canExecute || pending}
      >
        {`Prune ${type}`}
      </ConfirmModalWithDisable>
    );
  }
};
