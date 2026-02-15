import { useInvalidate, useWrite } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import ConfirmButton from "@/ui/confirm-button";
import { notifications } from "@mantine/notifications";

export default function DeleteTag({
  tagId,
  disabled,
}: {
  tagId: string;
  disabled: boolean;
}) {
  const invalidate = useInvalidate();
  const { mutate, isPending } = useWrite("DeleteTag", {
    onSuccess: () => {
      invalidate(["ListTags"]);
      notifications.show({ message: "Tag Deleted" });
    },
  });
  return (
    <ConfirmButton
      icon={<ICONS.Delete size="1rem" />}
      onClick={(e) => {
        e.stopPropagation();
        mutate({ id: tagId });
      }}
      loading={isPending}
      disabled={disabled}
    >
      Delete
    </ConfirmButton>
  );
}
