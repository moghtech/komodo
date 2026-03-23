import { useTags, useWrite } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import ConfirmButton from "@/ui/confirm-button";
import { notifications } from "@mantine/notifications";

export default function BatchDeleteAllTerminals({
  refetch,
  noTerminals,
}: {
  refetch: () => void;
  noTerminals: boolean;
}) {
  const { mutate, isPending } = useWrite("BatchDeleteAllTerminals", {
    onSuccess: () => {
      refetch();
      notifications.show({ message: "Deleted All Terminals" });
    },
  });
  const { tags } = useTags();
  return (
    <ConfirmButton
      icon={<ICONS.Delete size="1rem" />}
      w={160}
      onClick={() => mutate({ query: { tags } })}
      disabled={noTerminals}
      loading={isPending}
      confirmProps={{ variant: "filled", color: "red" }}
    >
      Delete All
    </ConfirmButton>
  );
}
