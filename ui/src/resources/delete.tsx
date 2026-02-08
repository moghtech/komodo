import { useNavigate } from "react-router-dom";
import { UsableResource } from ".";
import { useRead, useWrite } from "@/lib/hooks";
import { usableResourcePath } from "@/lib/utils";
import ConfirmModal from "@/ui/confirm-modal";
import { ICONS } from "@/theme/icons";

export default function DeleteResource({
  type,
  id,
}: {
  type: UsableResource;
  id: string;
}) {
  const nav = useNavigate();
  const key = type === "ResourceSync" ? "sync" : type.toLowerCase();
  const resource = useRead(`Get${type}`, {
    [key]: id,
  } as any).data;
  const { mutateAsync, isPending } = useWrite(`Delete${type}`, {
    onSuccess: () => nav(`/${usableResourcePath(type)}`),
  });

  if (!resource) return null;

  return (
    <ConfirmModal
      targetProps={{ color: "red" }}
      icon={<ICONS.Delete size="1rem" />}
      confirmText={resource.name}
      onConfirm={() => mutateAsync({ id })}
      loading={isPending}
    >
      Delete
    </ConfirmModal>
  );
}
