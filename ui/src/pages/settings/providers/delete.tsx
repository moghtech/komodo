import { useInvalidate, useWrite } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import ConfirmButton from "@/ui/confirm-button";
import { notifications } from "@mantine/notifications";

export default function DeleteProviderAccount({
  type,
  id,
}: {
  type: "GitProvider" | "DockerRegistry";
  id: string;
}) {
  const invalidate = useInvalidate();
  const { mutate, isPending } = useWrite(`Delete${type}Account`, {
    onSuccess: () => {
      invalidate([`List${type}Accounts`], [`Get${type}Account`]);
      notifications.show({ message: "Account deleted" });
    },
  });
  return (
    <ConfirmButton
      icon={<ICONS.Delete size="1rem" />}
      onClick={() => mutate({ id })}
      loading={isPending}
      confirmProps={{ variant: "filled", color: "red" }}
    >
      Delete
    </ConfirmButton>
  );
}
