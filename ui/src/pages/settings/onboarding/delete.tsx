import { useInvalidate, useWrite } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import ConfirmButton from "@/ui/confirm-button";
import { notifications } from "@mantine/notifications";

export default function DeleteOnboardingKey({
  publicKey,
}: {
  publicKey: string;
}) {
  const invalidate = useInvalidate();
  const { mutate, isPending } = useWrite("DeleteOnboardingKey", {
    onSuccess: () => {
      invalidate(["ListOnboardingKeys"]);
      notifications.show({ message: "Onboarding Key Deleted" });
    },
  });
  return (
    <ConfirmButton
      icon={<ICONS.Delete size="1rem" />}
      onClick={(e) => {
        e.stopPropagation();
        mutate({ public_key: publicKey });
      }}
      loading={isPending}
    >
      Delete
    </ConfirmButton>
  );
}
