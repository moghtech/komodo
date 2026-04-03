import { useNavigate } from "react-router-dom";
import { useRead, useUser, useWrite } from "@/lib/hooks";
import { ConfirmModal } from "mogh_ui";
import { ICONS } from "@/lib/icons";

export default function DeleteUserGroup({ groupId }: { groupId: string }) {
  const nav = useNavigate();
  const isAdmin = useUser().data?.admin;
  const group = useRead("GetUserGroup", {
    user_group: groupId,
  }).data;
  const { mutateAsync, isPending } = useWrite("DeleteUserGroup", {
    onSuccess: () => nav("/settings"),
  });

  if (!group || !isAdmin) return null;

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
      targetProps={{ variant: "filled", color: "red", w: "fit", px: "xs" }}
      confirmText={group?.name ?? ""}
      onConfirm={() => mutateAsync({ id: groupId })}
      loading={isPending}
    >
      <ICONS.Delete size="1.3rem" />
    </ConfirmModal>
  );
}
