import { ActionWithDialog, ConfirmButton } from "@components/util";
import { useExecute, useRead } from "@lib/hooks";
import { Scissors } from "lucide-react";
import { useServer } from ".";
import { has_minimum_permissions } from "@lib/utils";
import { Types } from "komodo_client";

export const Prune = ({
  server_id,
  type,
}: {
  server_id: string;
  type: "Containers" | "Networks" | "Images" | "Volumes" | "Buildx" | "System";
}) => {
  const server = useServer(server_id);
  const { mutate, isPending } = useExecute(`Prune${type}`);
  const action_state = useRead(
    "GetServerActionState",
    { server: server_id },
    { refetchInterval: 5000 }
  ).data;
  const perms = useRead("GetPermission", {
    target: { type: "Server", id: server_id },
  }).data;

  if (!server) return;

  const canExecute = has_minimum_permissions(
    perms,
    Types.PermissionLevel.Execute
  );

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

  const pending = isPending || action_state?.[pruningKey];

  if (type === "Images" || type === "Networks" || type === "Buildx") {
    return (
      <ConfirmButton
        title={`Prune ${type}`}
        icon={<Scissors className="w-4 h-4" />}
        onClick={() => mutate({ server: server_id })}
        loading={pending}
        disabled={!canExecute || pending}
      />
    );
  } else {
    return (
      <ActionWithDialog
        name={server?.name}
        title={`Prune ${type}`}
        icon={<Scissors className="w-4 h-4" />}
        onClick={() => mutate({ server: server_id })}
        loading={pending}
        disabled={!canExecute || pending}
      />
    );
  }
};
