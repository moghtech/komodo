import { useInvalidate, useWrite } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import ConfirmModal from "@/ui/confirm-modal";
import { notifications } from "@mantine/notifications";
import { Types } from "komodo_client";
import { useNavigate } from "react-router-dom";

export default function DeleteUserGroup({ group }: { group: Types.UserGroup }) {
  const nav = useNavigate();
  const inv = useInvalidate();
  const { mutateAsync: deleteGroup, isPending } = useWrite("DeleteUserGroup", {
    onSuccess: () => {
      inv(
        ["ListUserGroups"],
        ["GetUserGroup", { user_group: group._id?.$oid! }],
      );
      notifications.show({
        message: `Deleted User Group ${group.name}`,
        color: "green",
      });
      nav("/settings");
    },
  });

  return (
    <ConfirmModal
      targetProps={{ variant: "filled", color: "red" }}
      icon={<ICONS.Delete size="1rem" />}
      onConfirm={() => deleteGroup({ id: group._id?.$oid! })}
      confirmText={group.name}
      loading={isPending}
    >
      Delete
    </ConfirmModal>
  );
}
