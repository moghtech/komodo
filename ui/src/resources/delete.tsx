import { useNavigate } from "react-router-dom";
import { UsableResource } from ".";
import { usePermissions, useRead, useWrite } from "@/lib/hooks";
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
  const { canWrite } = usePermissions({ type, id });
  const resource = useRead(`Get${type}`, {
    [key]: id,
  } as any).data;
  const { mutateAsync, isPending } = useWrite(`Delete${type}`, {
    onSuccess: () => nav(`/${usableResourcePath(type)}`),
  });

  if (!resource || !canWrite) return null;

  return (
    <ConfirmModal
      title={
        <>
          Confirm <b>Delete</b>
        </>
      }
      confirmButtonContent="Delete"
      icon={<ICONS.Delete size="1rem" />}
      targetNoIcon
      targetProps={{ w: "fit", px: "xs" }}
      confirmText={resource.name}
      onConfirm={() => mutateAsync({ id })}
      loading={isPending}
      confirmProps={{ variant: "filled", color: "red" }}
    >
      <ICONS.Delete size="1.3rem" />
    </ConfirmModal>
  );
}
