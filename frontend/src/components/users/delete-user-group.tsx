import { ActionWithDialog } from "@components/util";
import { useInvalidate, useWrite } from "@lib/hooks";
import { useToast } from "@ui/use-toast";
import { Types } from "komodo_client";
import { Trash } from "lucide-react";
import { useNavigate } from "react-router-dom";

export const DeleteUserGroup = ({ group }: { group: Types.UserGroup }) => {
  const nav = useNavigate();
  const inv = useInvalidate();
  const { toast } = useToast();
  const { mutate, isPending } = useWrite("DeleteUserGroup", {
    onSuccess: () => {
      inv(
        ["ListUserGroups"],
        ["GetUserGroup", { user_group: group._id?.$oid! }]
      );
      toast({ title: `Deleted User Group ${group.name}` });
      nav("/settings");
    },
  });

  return (
    <ActionWithDialog
      name={group.name}
      title="Delete"
      icon={<Trash className="h-4 w-4" />}
      variant="destructive"
      onClick={() => mutate({ id: group._id?.$oid! })}
      disabled={isPending}
      loading={isPending}
    />
  );
};
