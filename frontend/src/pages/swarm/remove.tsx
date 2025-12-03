import { ActionWithDialog } from "@components/util";
import { useExecute } from "@lib/hooks";
import { Trash } from "lucide-react";
import { useNavigate } from "react-router-dom";

export type RemovableSwarmResourceType =
  | "Node"
  | "Stack"
  | "Service"
  | "Config"
  | "Secret";

export const RemoveSwarmResource = ({
  id,
  type,
  resource_id,
  resource_name,
}: {
  id: string;
  type: RemovableSwarmResourceType;
  resource_id: string;
  resource_name?: string;
}) => {
  const nav = useNavigate();
  const { mutate: remove, isPending } = useExecute(`RemoveSwarm${type}s`, {
    onSuccess: () => nav("/swarms/" + id),
  });
  let key = `${type.toLowerCase()}s`;
  return (
    <ActionWithDialog
      name={resource_name ?? resource_id}
      title="Remove"
      icon={<Trash className="h-4 w-4" />}
      onClick={() =>
        remove({ swarm: id, [key]: [resource_id], detach: false } as any)
      }
      disabled={isPending}
      loading={isPending}
    />
  );
};
